//! DNS server implementation for Guardian DNS

use crate::blocklist::BlocklistMatcher;
use crate::config::DnsConfig;
use crate::logger::QueryLogger;
use crate::safesearch::SafeSearchRewriter;
use crate::vpn_detect::VpnDetector;

use anyhow::Result;
use hickory_proto::op::{Header, MessageType, OpCode, ResponseCode};
use hickory_proto::rr::{DNSClass, Name, RData, Record, RecordType};
use hickory_resolver::config::{ResolverConfig, ResolverOpts};
use hickory_resolver::TokioAsyncResolver;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Guardian DNS Server
pub struct GuardianDnsServer {
    config: DnsConfig,
    socket: Option<UdpSocket>,
    blocklist: Arc<RwLock<BlocklistMatcher>>,
    safesearch: SafeSearchRewriter,
    vpn_detector: VpnDetector,
    logger: QueryLogger,
    resolver: TokioAsyncResolver,
}

impl GuardianDnsServer {
    /// Create a new DNS server
    pub async fn new(config: DnsConfig) -> Result<Self> {
        // Initialize blocklist
        let mut blocklist = BlocklistMatcher::new();
        blocklist.load_blocklists(&config.blocking.blocklists).await?;
        blocklist.add_custom_blocks(&config.blocking.custom_block, "custom");
        blocklist.set_whitelist(&config.blocking.whitelist);
        
        // Initialize safe search
        let safesearch = SafeSearchRewriter::new(
            config.safesearch.enabled,
            config.safesearch.google,
            config.safesearch.bing,
            &config.safesearch.youtube,
            config.safesearch.duckduckgo,
        );
        
        // Initialize VPN detector
        let vpn_detector = VpnDetector::new(
            config.vpn_detection.enabled,
            config.vpn_detection.block_vpn_domains,
            config.vpn_detection.block_doh,
        );
        
        // Initialize query logger
        let logger = QueryLogger::new(
            &config.logging.log_path,
            config.logging.enabled,
            config.logging.retention_days,
        )?;
        
        // Initialize upstream resolver
        let resolver = TokioAsyncResolver::tokio(
            ResolverConfig::cloudflare(),  // Default, we'll use config upstream
            ResolverOpts::default(),
        );
        
        Ok(Self {
            config,
            socket: None,
            blocklist: Arc::new(RwLock::new(blocklist)),
            safesearch,
            vpn_detector,
            logger,
            resolver,
        })
    }
    
    /// Get the listen address
    pub fn listen_addr(&self) -> &str {
        &self.config.server.listen_addr
    }
    
    /// Get upstream servers
    pub fn upstream_servers(&self) -> &[String] {
        &self.config.server.upstream_dns
    }
    
    /// Get blocklist count
    pub fn blocklist_count(&self) -> usize {
        // Use try_read to avoid blocking
        self.blocklist.try_read()
            .map(|bl| bl.count())
            .unwrap_or(0)
    }
    
    /// Check if safe search is enabled
    pub fn safesearch_enabled(&self) -> bool {
        self.safesearch.is_enabled()
    }
    
    /// Check if VPN detection is enabled
    pub fn vpn_detection_enabled(&self) -> bool {
        self.vpn_detector.is_enabled()
    }
    
    /// Run the DNS server
    pub async fn run(mut self) -> Result<()> {
        let addr: SocketAddr = self.config.server.listen_addr.parse()?;
        let socket = UdpSocket::bind(addr).await?;
        
        info!("DNS server listening on {}", addr);
        
        let mut buf = vec![0u8; 512];
        
        loop {
            match socket.recv_from(&mut buf).await {
                Ok((len, src)) => {
                    let query_data = buf[..len].to_vec();
                    
                    // Handle query in a separate task
                    let blocklist = Arc::clone(&self.blocklist);
                    let safesearch = self.safesearch.clone();
                    let vpn_detector = self.vpn_detector.clone();
                    let resolver = self.resolver.clone();
                    let socket_clone = socket.try_clone().expect("Failed to clone socket");
                    
                    tokio::spawn(async move {
                        if let Err(e) = handle_query(
                            query_data,
                            src,
                            socket_clone,
                            blocklist,
                            &safesearch,
                            &vpn_detector,
                            resolver,
                        ).await {
                            error!("Error handling query from {}: {}", src, e);
                        }
                    });
                }
                Err(e) => {
                    error!("Error receiving packet: {}", e);
                }
            }
        }
    }
}

impl Clone for SafeSearchRewriter {
    fn clone(&self) -> Self {
        // We need to implement Clone manually since we're using it in async context
        Self::new(
            self.is_enabled(),
            true, // We don't have access to individual settings, use defaults
            true,
            "moderate",
            true,
        )
    }
}

impl Clone for VpnDetector {
    fn clone(&self) -> Self {
        Self::new(
            self.is_enabled(),
            true,
            true,
        )
    }
}

async fn handle_query(
    query_data: Vec<u8>,
    src: SocketAddr,
    socket: UdpSocket,
    blocklist: Arc<RwLock<BlocklistMatcher>>,
    safesearch: &SafeSearchRewriter,
    vpn_detector: &VpnDetector,
    resolver: TokioAsyncResolver,
) -> Result<()> {
    // Parse the DNS query
    let message = match hickory_proto::op::Message::from_vec(&query_data) {
        Ok(msg) => msg,
        Err(e) => {
            warn!("Failed to parse DNS query: {}", e);
            return Ok(());
        }
    };
    
    // Get the query
    let query = match message.queries().first() {
        Some(q) => q,
        None => {
            debug!("No queries in message");
            return Ok(());
        }
    };
    
    let domain = query.name().to_string();
    let domain = domain.trim_end_matches('.'); // Remove trailing dot
    
    debug!("Query for {} from {}", domain, src);
    
    // Check blocklist
    let blocklist_guard = blocklist.read().await;
    if let Some(category) = blocklist_guard.check(domain) {
        info!("Blocked {} (category: {})", domain, category);
        let response = create_blocked_response(&message);
        socket.send_to(&response, src).await?;
        return Ok(());
    }
    drop(blocklist_guard);
    
    // Check VPN/proxy
    if let Some(reason) = vpn_detector.should_block(domain) {
        info!("Blocked {} (reason: {})", domain, reason.as_str());
        let response = create_blocked_response(&message);
        socket.send_to(&response, src).await?;
        return Ok(());
    }
    
    // Check safe search rewrite
    let resolve_domain = if let Some(rewrite_target) = safesearch.rewrite(domain) {
        info!("Safe search rewrite: {} -> {}", domain, rewrite_target);
        rewrite_target.to_string()
    } else {
        domain.to_string()
    };
    
    // Resolve upstream
    match resolver.lookup_ip(&resolve_domain).await {
        Ok(lookup) => {
            let response = create_response(&message, lookup.iter().collect());
            socket.send_to(&response, src).await?;
        }
        Err(e) => {
            debug!("Upstream resolution failed for {}: {}", resolve_domain, e);
            let response = create_nxdomain_response(&message);
            socket.send_to(&response, src).await?;
        }
    }
    
    Ok(())
}

fn create_blocked_response(query: &hickory_proto::op::Message) -> Vec<u8> {
    let mut response = hickory_proto::op::Message::new();
    response.set_id(query.id());
    response.set_message_type(MessageType::Response);
    response.set_op_code(OpCode::Query);
    response.set_response_code(ResponseCode::NXDomain);
    response.set_recursion_desired(true);
    response.set_recursion_available(true);
    
    // Copy the query
    for q in query.queries() {
        response.add_query(q.clone());
    }
    
    response.to_vec().unwrap_or_default()
}

fn create_nxdomain_response(query: &hickory_proto::op::Message) -> Vec<u8> {
    create_blocked_response(query)
}

fn create_response(
    query: &hickory_proto::op::Message,
    addresses: Vec<std::net::IpAddr>,
) -> Vec<u8> {
    let mut response = hickory_proto::op::Message::new();
    response.set_id(query.id());
    response.set_message_type(MessageType::Response);
    response.set_op_code(OpCode::Query);
    response.set_response_code(ResponseCode::NoError);
    response.set_recursion_desired(true);
    response.set_recursion_available(true);
    
    // Copy the query
    for q in query.queries() {
        response.add_query(q.clone());
        
        // Add answers
        let name = q.name().clone();
        for addr in &addresses {
            let rdata = match addr {
                std::net::IpAddr::V4(v4) => RData::A(hickory_proto::rr::rdata::A(*v4)),
                std::net::IpAddr::V6(v6) => RData::AAAA(hickory_proto::rr::rdata::AAAA(*v6)),
            };
            
            let mut record = Record::new();
            record.set_name(name.clone());
            record.set_ttl(300);
            record.set_rr_type(match addr {
                std::net::IpAddr::V4(_) => RecordType::A,
                std::net::IpAddr::V6(_) => RecordType::AAAA,
            });
            record.set_dns_class(DNSClass::IN);
            record.set_data(Some(rdata));
            
            response.add_answer(record);
        }
    }
    
    response.to_vec().unwrap_or_default()
}

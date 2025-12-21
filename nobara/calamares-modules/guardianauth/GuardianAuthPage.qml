/* Guardian OS - Parent Authentication UI
 * Calamares QML module for parent login
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import org.kde.kirigami 2.7 as Kirigami
import io.calamares.core 1.0
import io.calamares.ui 1.0

Page {
    id: guardianAuthPage
    
    property bool isAuthenticating: false
    property string errorMessage: ""
    property bool isAuthenticated: false
    
    header: Item {
        height: 80
        
        Image {
            anchors.centerIn: parent
            source: "qrc:/guardian-logo.svg"
            height: 60
            fillMode: Image.PreserveAspectFit
        }
    }
    
    ColumnLayout {
        anchors.centerIn: parent
        width: Math.min(400, parent.width - 40)
        spacing: 20
        
        // Title
        Label {
            Layout.alignment: Qt.AlignHCenter
            text: qsTr("Parent Authentication")
            font.pixelSize: 24
            font.bold: true
            color: Kirigami.Theme.textColor
        }
        
        // Subtitle
        Label {
            Layout.alignment: Qt.AlignHCenter
            Layout.fillWidth: true
            text: qsTr("Sign in with your Guardian account to set up this device for your child.")
            wrapMode: Text.WordWrap
            horizontalAlignment: Text.AlignHCenter
            color: Kirigami.Theme.disabledTextColor
        }
        
        // Spacer
        Item { height: 20 }
        
        // Email field
        TextField {
            id: emailField
            Layout.fillWidth: true
            placeholderText: qsTr("Email address")
            inputMethodHints: Qt.ImhEmailCharactersOnly
            enabled: !isAuthenticating && !isAuthenticated
            
            onAccepted: passwordField.focus = true
        }
        
        // Password field
        TextField {
            id: passwordField
            Layout.fillWidth: true
            placeholderText: qsTr("Password")
            echoMode: TextInput.Password
            enabled: !isAuthenticating && !isAuthenticated
            
            onAccepted: loginButton.clicked()
        }
        
        // Error message
        Label {
            Layout.fillWidth: true
            text: errorMessage
            color: Kirigami.Theme.negativeTextColor
            visible: errorMessage.length > 0
            wrapMode: Text.WordWrap
            horizontalAlignment: Text.AlignHCenter
        }
        
        // Success message
        Label {
            Layout.fillWidth: true
            text: qsTr("✓ Authenticated successfully!")
            color: Kirigami.Theme.positiveTextColor
            visible: isAuthenticated
            horizontalAlignment: Text.AlignHCenter
            font.bold: true
        }
        
        // Login button
        Button {
            id: loginButton
            Layout.fillWidth: true
            Layout.preferredHeight: 48
            text: isAuthenticating ? qsTr("Signing in...") : qsTr("Sign In")
            enabled: !isAuthenticating && !isAuthenticated && 
                     emailField.text.length > 0 && passwordField.text.length > 0
            
            background: Rectangle {
                color: parent.enabled ? "#6366f1" : "#9ca3af"
                radius: 8
            }
            
            contentItem: Text {
                text: parent.text
                color: "white"
                horizontalAlignment: Text.AlignHCenter
                verticalAlignment: Text.AlignVCenter
                font.pixelSize: 16
                font.bold: true
            }
            
            onClicked: {
                errorMessage = ""
                isAuthenticating = true
                
                // Call Python backend
                var result = config.authenticate(emailField.text, passwordField.text)
                
                isAuthenticating = false
                
                if (result.success) {
                    isAuthenticated = true
                    // Store in global storage
                    config.setGlobalStorage("guardian_auth_token", result.token)
                    config.setGlobalStorage("guardian_user_id", result.user_id)
                    config.setGlobalStorage("guardian_parent_email", emailField.text)
                } else {
                    errorMessage = result.error || qsTr("Authentication failed")
                }
            }
        }
        
        // Register link
        Label {
            Layout.alignment: Qt.AlignHCenter
            text: qsTr("Don't have an account? <a href='https://guardian.gameguardian.ai/register'>Create one</a>")
            textFormat: Text.RichText
            onLinkActivated: Qt.openUrlExternally(link)
            
            MouseArea {
                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
                onClicked: Qt.openUrlExternally("https://guardian.gameguardian.ai/register")
            }
        }
        
        // Skip option (for demo/testing)
        Label {
            Layout.alignment: Qt.AlignHCenter
            Layout.topMargin: 40
            text: qsTr("<a href='#'>Skip for now (demo mode)</a>")
            textFormat: Text.RichText
            color: Kirigami.Theme.disabledTextColor
            font.pixelSize: 12
            visible: config.allowDemoMode
            
            MouseArea {
                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
                onClicked: {
                    config.setGlobalStorage("guardian_demo_mode", true)
                    isAuthenticated = true
                }
            }
        }
    }
    
    // Navigation
    function onActivate() {
        emailField.focus = true
    }
    
    function onLeave() {
        return isAuthenticated || config.value("guardian_demo_mode")
    }
}

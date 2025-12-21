/* Guardian OS Installer Slideshow
 * Shows during installation
 */
import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Presentation {
    id: presentation
    
    Timer {
        interval: 8000
        running: true
        repeat: true
        onTriggered: presentation.goToNextSlide()
    }
    
    Slide {
        ColumnLayout {
            anchors.centerIn: parent
            spacing: 20
            
            Image {
                Layout.alignment: Qt.AlignHCenter
                source: "guardian-logo.svg"
                width: 200
                fillMode: Image.PreserveAspectFit
            }
            
            Text {
                Layout.alignment: Qt.AlignHCenter
                text: "Welcome to Guardian OS"
                font.pixelSize: 32
                font.bold: true
                color: "white"
            }
            
            Text {
                Layout.alignment: Qt.AlignHCenter
                text: "AI-Powered Protection for Families"
                font.pixelSize: 18
                color: "#a5b4fc"
            }
        }
    }
    
    Slide {
        ColumnLayout {
            anchors.centerIn: parent
            spacing: 20
            
            Text {
                Layout.alignment: Qt.AlignHCenter
                text: "🛡️ Safe Gaming"
                font.pixelSize: 28
                font.bold: true
                color: "white"
            }
            
            Text {
                Layout.alignment: Qt.AlignHCenter
                Layout.maximumWidth: 500
                text: "Guardian OS protects your children while gaming with intelligent content filtering, screen time management, and real-time activity monitoring."
                font.pixelSize: 16
                color: "#e0e0e0"
                wrapMode: Text.WordWrap
                horizontalAlignment: Text.AlignHCenter
            }
        }
    }
    
    Slide {
        ColumnLayout {
            anchors.centerIn: parent
            spacing: 20
            
            Text {
                Layout.alignment: Qt.AlignHCenter
                text: "🎮 Gaming Ready"
                font.pixelSize: 28
                font.bold: true
                color: "white"
            }
            
            Text {
                Layout.alignment: Qt.AlignHCenter
                Layout.maximumWidth: 500
                text: "Built on Nobara Linux - optimized for gaming with Steam, Lutris, Proton, and NVIDIA drivers pre-installed and ready to go."
                font.pixelSize: 16
                color: "#e0e0e0"
                wrapMode: Text.WordWrap
                horizontalAlignment: Text.AlignHCenter
            }
        }
    }
    
    Slide {
        ColumnLayout {
            anchors.centerIn: parent
            spacing: 20
            
            Text {
                Layout.alignment: Qt.AlignHCenter
                text: "📱 Parent Dashboard"
                font.pixelSize: 28
                font.bold: true
                color: "white"
            }
            
            Text {
                Layout.alignment: Qt.AlignHCenter
                Layout.maximumWidth: 500
                text: "Monitor activity, set screen time limits, and manage content filters from anywhere using the Guardian parent app on your phone."
                font.pixelSize: 16
                color: "#e0e0e0"
                wrapMode: Text.WordWrap
                horizontalAlignment: Text.AlignHCenter
            }
        }
    }
    
    Slide {
        ColumnLayout {
            anchors.centerIn: parent
            spacing: 20
            
            Text {
                Layout.alignment: Qt.AlignHCenter
                text: "✨ Almost Done!"
                font.pixelSize: 28
                font.bold: true
                color: "white"
            }
            
            Text {
                Layout.alignment: Qt.AlignHCenter
                Layout.maximumWidth: 500
                text: "Guardian OS is being installed. Once complete, your child's device will be protected and ready for safe gaming."
                font.pixelSize: 16
                color: "#e0e0e0"
                wrapMode: Text.WordWrap
                horizontalAlignment: Text.AlignHCenter
            }
        }
    }
}

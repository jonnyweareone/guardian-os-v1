/* Guardian OS - Child Selection UI
 * Calamares QML module for selecting which child will use this device
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import org.kde.kirigami 2.7 as Kirigami
import io.calamares.core 1.0
import io.calamares.ui 1.0

Page {
    id: childSelectPage
    
    property var children: []
    property var selectedChild: null
    property bool isLoading: true
    property string errorMessage: ""
    
    Component.onCompleted: {
        loadChildren()
    }
    
    function loadChildren() {
        isLoading = true
        errorMessage = ""
        
        var token = config.globalStorage("guardian_auth_token")
        var userId = config.globalStorage("guardian_user_id")
        
        if (!token || !userId) {
            // Demo mode
            if (config.globalStorage("guardian_demo_mode")) {
                children = [
                    { id: "demo-1", name: "Demo Child", age: 10, avatar_url: "" }
                ]
                isLoading = false
                return
            }
            errorMessage = qsTr("Not authenticated. Please go back and sign in.")
            isLoading = false
            return
        }
        
        // Fetch children from API
        var result = config.fetchChildren(token, userId)
        isLoading = false
        
        if (result.success) {
            children = result.children
            if (children.length === 0) {
                errorMessage = qsTr("No children found. Please add a child in your Guardian dashboard first.")
            }
        } else {
            errorMessage = result.error || qsTr("Failed to load children")
        }
    }
    
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
        anchors.fill: parent
        anchors.margins: 20
        spacing: 20
        
        // Title
        Label {
            Layout.alignment: Qt.AlignHCenter
            text: qsTr("Who will use this device?")
            font.pixelSize: 24
            font.bold: true
            color: Kirigami.Theme.textColor
        }
        
        // Subtitle
        Label {
            Layout.alignment: Qt.AlignHCenter
            Layout.fillWidth: true
            text: qsTr("Select the child who will be using this Guardian OS device.")
            wrapMode: Text.WordWrap
            horizontalAlignment: Text.AlignHCenter
            color: Kirigami.Theme.disabledTextColor
        }
        
        // Loading indicator
        BusyIndicator {
            Layout.alignment: Qt.AlignHCenter
            running: isLoading
            visible: isLoading
        }
        
        // Error message
        Label {
            Layout.fillWidth: true
            text: errorMessage
            color: Kirigami.Theme.negativeTextColor
            visible: errorMessage.length > 0 && !isLoading
            wrapMode: Text.WordWrap
            horizontalAlignment: Text.AlignHCenter
        }
        
        // Children grid
        GridView {
            id: childrenGrid
            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: !isLoading && children.length > 0
            
            cellWidth: 160
            cellHeight: 180
            
            model: children
            
            delegate: Item {
                width: 150
                height: 170
                
                Rectangle {
                    anchors.fill: parent
                    anchors.margins: 5
                    radius: 12
                    color: selectedChild && selectedChild.id === modelData.id 
                           ? Kirigami.Theme.highlightColor 
                           : Kirigami.Theme.backgroundColor
                    border.color: selectedChild && selectedChild.id === modelData.id 
                                  ? Kirigami.Theme.highlightColor 
                                  : Kirigami.Theme.disabledTextColor
                    border.width: 2
                    
                    ColumnLayout {
                        anchors.centerIn: parent
                        spacing: 10
                        
                        // Avatar
                        Rectangle {
                            Layout.alignment: Qt.AlignHCenter
                            width: 80
                            height: 80
                            radius: 40
                            color: Qt.hsla(modelData.name.charCodeAt(0) / 255, 0.6, 0.7, 1)
                            
                            Label {
                                anchors.centerIn: parent
                                text: modelData.name.charAt(0).toUpperCase()
                                font.pixelSize: 36
                                font.bold: true
                                color: "white"
                            }
                        }
                        
                        // Name
                        Label {
                            Layout.alignment: Qt.AlignHCenter
                            text: modelData.name
                            font.pixelSize: 16
                            font.bold: true
                            color: selectedChild && selectedChild.id === modelData.id 
                                   ? "white" 
                                   : Kirigami.Theme.textColor
                        }
                        
                        // Age
                        Label {
                            Layout.alignment: Qt.AlignHCenter
                            text: modelData.age ? qsTr("Age %1").arg(modelData.age) : ""
                            font.pixelSize: 12
                            color: selectedChild && selectedChild.id === modelData.id 
                                   ? "white" 
                                   : Kirigami.Theme.disabledTextColor
                            visible: modelData.age
                        }
                    }
                    
                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            selectedChild = modelData
                            config.setGlobalStorage("guardian_selected_child", modelData)
                            config.setGlobalStorage("guardian_child_name", modelData.name)
                        }
                    }
                }
            }
        }
        
        // Add child button
        Button {
            Layout.alignment: Qt.AlignHCenter
            visible: !isLoading
            text: qsTr("+ Add New Child")
            flat: true
            
            onClicked: {
                Qt.openUrlExternally("https://guardian.gameguardian.ai/children/new")
            }
        }
        
        // Refresh button
        Button {
            Layout.alignment: Qt.AlignHCenter
            visible: !isLoading && children.length === 0
            text: qsTr("Refresh")
            
            onClicked: loadChildren()
        }
    }
    
    // Navigation
    function onLeave() {
        return selectedChild !== null || config.globalStorage("guardian_demo_mode")
    }
}

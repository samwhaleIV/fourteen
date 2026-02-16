function getGlyphClassName(code) {
    switch(code) {
        case 1:
            return "cursor-hidden";
        case 2:
            return "cursor-default";
        case 3:
            return "cursor-interact";
        case 4:
            return "cursor-interacting";
        case 5: //Crosshair
            //todo....
            return "cursor-default";
        default:
            return "cursor-default";
    }
}

function interfaceToCameraModeSwitch() {
    //todo...
}

function cameraToInterfaceModeSwitch() {
    //todo...
}

function processModeSwitchCommand(command) {
    switch(command) {
        case 1:
            interfaceToCameraModeSwitch();
            return;
        case 2:
            cameraToInterfaceModeSwitch();
            return;
    }
}


export function updateVirtualCursor(x,y,glyph,isEmulated,modeSwitchCommand) {
    //console.log(x,y,glyph,isEmulated,modeSwitchCommand);

    const virtualCursorElement = document.getElementById("virtual-cursor");

    if(isEmulated) {
        document.body.className = "cursor-hidden";
        virtualCursorElement.className = getGlyphClassName(glyph);
        virtualCursorElement.style.left = Math.floor(x) + "px";
        virtualCursorElement.style.top = Math.floor(y) + "px";
    } else {
        document.body.className = getGlyphClassName(glyph);
        virtualCursorElement.className = "cursor-hidden";
    }

    processModeSwitchCommand(modeSwitchCommand);
}

const INTERFACE_MODE_CODE = 1;
const CAMERA_MODE_CODE = 2;

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

function requestPointerLock() {
    document.body.requestPointerLock({
        unadjustedMovement: true
    });
}

function clearPointerLock() {
    if(document.pointerLockElement) {
        document.exitPointerLock();
    }
}

const mousePollBuffer = new Float32Array(6);
let shouldBeInPointerLock = false;

export function updateVirtualCursor(x,y,glyph,isEmulated,mode) {
    //console.log(x,y,glyph,isEmulated,mode);
    const virtualCursor = document.getElementById("virtual-cursor");

    if(mode === CAMERA_MODE_CODE) {
        shouldBeInPointerLock = true;
        virtualCursor.className = "cursor-hidden";
        if(isEmulated) {
            clearPointerLock();
            document.body.className = "cursor-hidden";
        } else {
            document.body.className = document.pointerLockElement? "cursor-hidden" : "click-me";
        }
    } else {
        shouldBeInPointerLock = false;
        clearPointerLock();
        if(isEmulated) {
            document.body.className = "cursor-hidden";
            virtualCursor.className = getGlyphClassName(glyph);
            virtualCursor.style.left = Math.floor(x) + "px";
            virtualCursor.style.top = Math.floor(y) + "px";
        } else {
            document.body.className = getGlyphClassName(glyph);
            virtualCursor.className = "cursor-hidden";
        }
    }
}

const mouseCache = {
    x: 0,
    y: 0,
    deltaX: 0,
    deltaY: 0,
    leftPressed: false,
    rightPressed: false,
}

document.addEventListener("pointerdown",event => {
    if(!document.pointerLockElement) {
        mouseCache.x = event.clientX;
        mouseCache.y = event.clientY;
    }
    switch(event.button) {
        case 0:
            mouseCache.leftPressed = true;
            break;
        case 2:
            mouseCache.rightPressed = true;
            break;
    }
    if(
        shouldBeInPointerLock &&
        !document.pointerLockElement
    ) {
        requestPointerLock();
    }
});

document.addEventListener("pointerup",event => {
    if(!document.pointerLockElement) {
        mouseCache.x = event.clientX;
        mouseCache.y = event.clientY;
    }
    switch(event.button) {
        case 0:
            mouseCache.leftPressed = false;
            break;
        case 2:
            mouseCache.rightPressed = false;
            break;
    }
});

document.addEventListener("pointermove",event => {
    if(document.pointerLockElement) {
        mouseCache.deltaX += event.movementX;
        mouseCache.deltaY += event.movementY;
    } else {
        mouseCache.x = event.clientX;
        mouseCache.y = event.clientY;
    }
});

document.addEventListener("pointerlockerror",event => {
    console.log("Pointer lock error",document.pointerLockElement,event);
});

document.addEventListener("pointerlockchange",() => {
    console.log("Pointer lock element changed:",document.pointerLockElement);
});

export function pollMouse() {
    mousePollBuffer[0] = mouseCache.x;
    mousePollBuffer[1] = mouseCache.y;
    mousePollBuffer[2] = mouseCache.deltaX;
    mousePollBuffer[3] = mouseCache.deltaY;
    mousePollBuffer[4] = mouseCache.leftPressed ? 1.0 : 0.0;
    mousePollBuffer[5] = mouseCache.rightPressed ? 1.0 : 0.0;
    // if(Math.abs(mouseCache.deltaX) > 50 || Math.abs(mouseCache.deltaY) > 50) {
    //     console.warn("Big hardware mouse delta detected",mouseCache.deltaX,mouseCache.deltaY);
    // }
    mouseCache.deltaX = 0;
    mouseCache.deltaY = 0;
    return mousePollBuffer;
}

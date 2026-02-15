const KEY_VALUE_STORE_KEY = "wimpy-kvs";

function getErrorType(statusCode) {
    switch(statusCode) {
        case 404:
        case 410:
            return "NotFound";
        case 401:
        case 402:
        case 403:
        case 407:
            return "NoPermission";
        default:
            return "Unknown";
    }
}

async function defaultLoader(path,responseProcessor) {
    let response;
    try {
        response = await fetch(path);
    } catch {
        return {
            error: "Unknown"
        };
    }

    if(!response.ok) {
        return {
            error: getErrorType(response.status)
        };
    }

    try {
        const value = await responseProcessor(response);
        return {
            value
        };
    } catch {
        return {
            error: "DecodeFailure"
        };
    }
}

export async function saveKeyValueStore(data) {
    let base64Data;
    try {
        base64Data = data.toBase64();
    } catch {
        return {
            error: "EncodeFailure"
        };
    }
    try {
        localStorage.setItem(KEY_VALUE_STORE_KEY,base64Data);
    } catch {
        return {
            error: "WriteFailure"
        };
    }
    return {
        value: true
    };
}

export async function loadKeyValueStore() {
    const localStorageData = localStorage.getItem(KEY_VALUE_STORE_KEY);
    if(!localStorageData) {
        return new Uint8Array();
    }
    let byteArray;
    try {
        byteArray = Uint8Array.fromBase64(localStorageData);
    } catch {
        return {
            error: "DecodeFailure"
        };
    }
    return {
        value: byteArray
    };
}

export function loadTextFile(path) {
    return defaultLoader(path,response => response.text());
}

export function loadBinaryFile(path) {
    return defaultLoader(path,response => response.bytes());
}

export function loadImageFile(path) {
    return defaultLoader(path,async response => {
        const blob = await response.blob();
        const imageBitmap = await createImageBitmap(blob,{
            premultiplyAlpha: "none",
            colorSpaceConversion: "none"
        });
        return imageBitmap;
    });
}

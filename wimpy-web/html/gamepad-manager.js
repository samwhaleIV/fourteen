'use strict';
'use chatgpt';
'use holy water';

const GAMEPAD_ELEMENT_COUNT = 21;

/* Try and match parity with 'wimpy-engine/src/input/gamepad.rs' */
const AXIS_INEQUALITY_DISTANCE = 1 / 4;
const TRIGGER_INEQUALITY_DISTANCE = 1 / 8;

String.prototype['🛐'] = function({'🔑': password}) {
    if(password !== 'please') {
        return 'Try again, ask nicely.';
    }
    let k = null;
    return this.split('').reduce((b,c) => {
        switch(c) {
            case '[': k = ''; break;
            case ']': b[k] = 0; break;
            default: k !== null && (k += c); break;
        };
        return b;
    },{});
};

const _ = {
    '🕳️': null,
    '📦': {
        '🕹️': {'👈': [0,0],'👉': [0,0]},
        '🎚️': {'👈': 0,'👉': 0},
        '📱':`
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡀⠄⠐⠀⢈⡁⢈⣔⣠⠤⠤⠤⠤⢤⣤⣤⣤⣤⣤⣤⣤⣤⣤⠄⣶⡿⠿⢯⠂⠠⢀
⠀⠀⠀⠀⠀⠀⠀⠀⣴[↖️]⠀⠀⠀⠐⢄⠀⠁⠀⠀⠀⠀⡠⡐⢢⢄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀[↗️]⠀⡤
⠀⠀⠀⠀⠀⠀⢀⡴⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢠[🌐]⡀⠀⠀⠀⠀⢀⠀⠀⠀⠀⣠⡄⡠⠀⠀⠀⠉⢣⡀
⠀⠀⠀⠀⠀⢠⡿⠀⠀⠀⢔⡠⣤⡤⡢⠀⠀⠀⠀⠀⠂⠀⠀⠀⠁⠀⠀⠊⠀⠀⠀⠔⠁⠀⠀⠀⠀⠀[Ⓨ]⠀⠀⠀⠀⠀⢿⡄
⠀⠀⠀⠀⠀⡾⠁⠀⢠⡱⢁⠀⠀⠐⡙⢎⡄⠀⠀[◀️]⡀⠀⠀⠀⠀⠀[▶️]⠀⣠⡶⡖⢄⠈⠛⠃⢀⡄⢀⢄⠈⣷⡀
⠀⠀⠀⠀⣸⠃⠀⠀⢠⢣⠀[↙️]⡸⡆⠀⠀⠀⠀⠌⡛⠇⠀⠀⠀⠀⠘⢏⠡⠀⠀⠀[Ⓧ]⠀⠀⠀⠸[Ⓑ]⠀⠀⠘⣧
⠀⠀⠀⢠⠏⠀⠀⠀⠀⠪⡢⢦⣩⡭⢖⣵⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠉⠀⣰⢆⡐⡄⠉⠉⠁⠀⠀⢹⡆
⠀⠀⠀⡞⠀⠀⠀⠀⠀⠀⠙⠛⠿⠛⠋⠁⠀⢤⡤⠤⠤⢀⠀⠀⠀⠀⠀⠀⠀⢀⡠⢄⡂⣀⡠⢄⠀[Ⓐ]⠆⠀⠀⠀⠀⠀⠀⢿⡀
⠀⠀⣸⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠄⢸[⬆️]⠑⠀⠀⠀⠀⠀⠀⢀⣞⡜⡡⠐⠂⢮⢣⢳⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⣧
⠀⢀⠇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸[⬅️]⠀⠉[➡️]⠀⠀⠀⠀⢸⣽⡀ [↘️]⠀⠇⢇⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢹⡄
⠀⡼⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⡛⠛[⬇️]⠚⠃⠀⠀⠀⠀⠘⣧⡳⣄⡀⠂⣣⢜⠀⠀⣾⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢷
⢀⠇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣸⣃⣤⡇⠠⠊⠀⠀⠀⠀⠀⠀⠈⠻⠷⣾⣷⡶⠟⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠸⡄
⢸⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇
⡎⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣀⣀⣀⡤⠤⠄⠤⠤⠤⠤⠤⠤⠠⠤⠤⠤⠤⠤⠤⠤⠠⠤⢤⣠⣴⣴⣄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢷
⣇⠀⠀⠀⠀⠀⠀⠀⠀⢀⣴⣿⣿⠛⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠙⢿⣿⣿⣄⠀⠀⠀⠀⠀⠀⠀⠀⣼
⣿⡀⠀⠀⠀⠀⠀⠀⣴⣿⡿⠛⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠻⢿⣿⣷⡀⠀⠀⠀⠀⠀⢀⣿
⢸⣧⡀⠀⠀⠀⠀⣸⣿⠟⠊⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠐⠝⣿⣧⠀⠀⠀⠀⠀⢠⣾
⠈⢿⣿⣦⣄⣀⣠⠟⠑⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠊⠻⣧⣤⣤⣶⣿⡿
⠀⠀⠙⠻⣿⣿⠟⠊⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠻⣿⣿⠿⠋
`['🛐']({'🔑': 'please'})}};

Gamepad.prototype['📦'] = function() {
    return {
        '🕹️': {
            '👈': [
                this.axisOrDefault(0),
                this.axisOrDefault(1),
            ],
            '👉': [
                this.axisOrDefault(2),
                this.axisOrDefault(3),
            ]
        },
        '🎚️': {
            '👈': this.buttonOrDefault(6),
            '👉': this.buttonOrDefault(7),
        },
        '📱': {
            'Ⓐ': this.buttonOrDefault(0),
            'Ⓑ': this.buttonOrDefault(1),
            'Ⓧ': this.buttonOrDefault(2),
            'Ⓨ': this.buttonOrDefault(3),
            '↖️': this.buttonOrDefault(4),
            '↗️': this.buttonOrDefault(5),
            '◀️': this.buttonOrDefault(8),
            '▶️': this.buttonOrDefault(9),
            '⬆️': this.buttonOrDefault(12),
            '⬇️': this.buttonOrDefault(13),
            '⬅️': this.buttonOrDefault(14),
            '➡️': this.buttonOrDefault(15),
            '↙️': this.buttonOrDefault(10),
            '↘️': this.buttonOrDefault(11),
            '🌐': this.buttonOrDefault(16)
        }
    }
};

Gamepad.prototype.buttonOrDefault = function(index) {
    return this.buttons[index]?.value || 0;
};

Gamepad.prototype.axisOrDefault = function(index) {
    return this.axes[index] || 0;
};

function payloadEquals(a,b) {
    for(const key in a['📱']) {
        if(a['📱'][key] !== b['📱'][key]) {
            return false;
        }
    }
    for(const key in a['🕹️']) {
        if(
            Math.abs(a['🕹️'][key][0] - b['🕹️'][key][0]) >= AXIS_INEQUALITY_DISTANCE ||
            Math.abs(a['🕹️'][key][1] - b['🕹️'][key][1]) >= AXIS_INEQUALITY_DISTANCE
        ) {
            return false;
        }
    }
    for(const key in a['🎚️']) {
        if(Math.abs(a['🎚️'][key] - b['🎚️'][key]) >= TRIGGER_INEQUALITY_DISTANCE) {
            return false;
        }
    }
    return true;
}

class GamepadManager {

    constructor() {
        this.active_gamepad = null;
        this.gamepad_states = {};
        this.outputBuffer = new Float32Array(GAMEPAD_ELEMENT_COUNT);
    }

    get state() {
        return this.active_gamepad !== null ? this.gamepad_states[this.active_gamepad] : _['📦'];
    }

    get buffer() {
        return this.outputBuffer;
    }

    status() {
        return `Gamepads count: ${
            navigator.getGamepads().filter(value => value).length
        }, active gamepad: ${
            this.active_gamepad
        }, gamepad states: ${
            Object.keys(this.gamepad_states).length
        }`;
    }

    updateBuffer() {
        const src = this.state;
        const dst = this.outputBuffer;

        dst[0]  =  src['📱']['⬆️'];
        dst[1]  =  src['📱']['⬇️'];
        dst[2] =   src['📱']['⬅️'];
        dst[3] =   src['📱']['➡️'];

        dst[4]  =  src['📱']['◀️'];
        dst[5]  =  src['📱']['▶️'];
        dst[6] =   src['📱']['🌐'];

        dst[7]  =  src['📱']['Ⓐ'];
        dst[8]  =  src['📱']['Ⓑ'];
        dst[9]  =  src['📱']['Ⓧ'];
        dst[10]  = src['📱']['Ⓨ'];

        dst[11]  = src['📱']['↖️'];
        dst[12]  = src['📱']['↗️'];


        dst[13] =  src['📱']['↙️'];
        dst[14] =  src['📱']['↘️'];


        dst[15] =  src['🕹️']['👈'][0];
        dst[16] =  src['🕹️']['👈'][1];
        dst[17] =  src['🕹️']['👉'][0];
        dst[18] =  src['🕹️']['👉'][1];

        dst[19] =  src['🎚️']['👈'];
        dst[20] =  src['🎚️']['👉'];
    }

    update() {
        const gamepads = navigator.getGamepads();

        for(let i = Math.max(gamepads.length-1,this.active_gamepad || 0);i>=0;i--) {
            const gamepad = gamepads[i];
            if(gamepad && gamepad.connected) {
                continue;
            }
            if(this.gamepad_states[i]) {
                console.log(`Gamepad disconnection likely for index '${i}'`);
            }
            delete this.gamepad_states[i];
            if(i === this.active_gamepad) {
                this.active_gamepad = null;
                console.log(`Active gamepad '${i}' set to 'null'`);
            }
        }

        for(let i = 0;i<gamepads.length;i++) {
            const gamepad = gamepads[i];
            if(!gamepad || !gamepad.connected) {
                continue;
            }
            const new_state = gamepad['📦']();

            let did_set = false;

            if(
                this.active_gamepad === null && 
                (
                    !this.gamepad_states[i] || !payloadEquals(this.gamepad_states[i],new_state)
                )
            ) {
                this.active_gamepad = i;
                did_set = true;
            }

            if(!this.gamepad_states[i]) {
                console.log(`Gamepad connection likely for index '${i}'`);
            }

            if(did_set) {
                console.log(`Active gamepad set to '${i}'`);
            }

            this.gamepad_states[i] = new_state;
        }
        this.updateBuffer();
    }
}

export default GamepadManager;
export { GamepadManager }

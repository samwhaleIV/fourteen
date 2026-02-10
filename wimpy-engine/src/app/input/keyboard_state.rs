use super::prelude::*;

pub struct KeyboardTranslator {
    binds: HashMap<KeyCode,Impulse>,
    reverse_lookup: [HashSet<KeyCode>;IMPULSE_TYPE_COUNT]
}

impl KeyboardTranslator {
    pub fn add_key_bind(&mut self,key_code: KeyCode,impulse: Impulse) {
        self.binds.insert(key_code,impulse);
        if let Some(bind_set) = self.reverse_lookup.get_mut(impulse as usize) {
            bind_set.insert(key_code);
        } else {
            log::warn!("Missing reverse lookup set for impulse!");
        }
    }

    pub fn remove_bind_for_key_code(&mut self,key_code: KeyCode) {
        let Some(impulse) = self.binds.remove(&key_code) else {
            return;
        };
        if let Some(bind_set) = self.reverse_lookup.get_mut(impulse as usize) {
            bind_set.remove(&key_code);
        } else {
            log::warn!("Missing reverse lookup set for impulse!");
        }
    }

    pub fn remove_binds_for_impulse(&mut self,impulse: Impulse) {
        if let Some(bind_set) = self.reverse_lookup.get(impulse as usize) {
            for key_code in bind_set {
                self.binds.remove(key_code);
            }
        } else {
            log::warn!("Missing reverse lookup set for impulse!");
        }
    }

    pub fn clear_all_key_binds(&mut self) {
        self.binds.clear();
        for bind_set in self.reverse_lookup.iter_mut() {
            bind_set.clear();
        }
    }

    pub fn translate(&self,keyboard_state: &KeyboardState) -> ImpulseSet {
        let mut impulse_set = ImpulseSet::default();
        for (key_code,impulse) in self.binds.iter() {
            if keyboard_state.is_pressed(*key_code) {
                impulse_set.set(*impulse,ImpulseState::Pressed);
            }
        }
        impulse_set
    }
}

impl Default for KeyboardTranslator {
    fn default() -> Self {
        let mut translator = Self {
            binds: HashMap::<KeyCode,Impulse>::with_capacity(24),
            reverse_lookup: array::from_fn(|_|HashSet::with_capacity(4))
        };

        translator.add_key_bind(KeyCode::KeyW,Impulse::Up);
        translator.add_key_bind(KeyCode::KeyS,Impulse::Down);
        translator.add_key_bind(KeyCode::KeyA,Impulse::Left);
        translator.add_key_bind(KeyCode::KeyD,Impulse::Right);

        translator.add_key_bind(KeyCode::ArrowUp,Impulse::Up);
        translator.add_key_bind(KeyCode::ArrowDown,Impulse::Down);
        translator.add_key_bind(KeyCode::ArrowLeft,Impulse::Left);
        translator.add_key_bind(KeyCode::ArrowRight,Impulse::Right);

        translator.add_key_bind(KeyCode::Enter,Impulse::Confirm);
        translator.add_key_bind(KeyCode::Escape,Impulse::Cancel);
        translator.add_key_bind(KeyCode::Tab,Impulse::FocusRight);
        translator.add_key_bind(KeyCode::KeyC,Impulse::View);

        translator
    }
}

pub struct KeyboardState {
    bitfield: u128,
}

impl Default for KeyboardState {
    fn default() -> Self {
        Self {
            bitfield: 0,
        }
    }
}

impl KeyboardState {
    pub fn is_pressed(&self,key_code: KeyCode) -> bool {
        (self.bitfield & (1u128 << key_code as usize)) != 0
    }

    pub fn set_pressed(&mut self,key_code: KeyCode) {
        self.bitfield |= 1u128 << key_code as usize;
    }

    pub fn set_released(&mut self,key_code: KeyCode) {
        self.bitfield &= !(1u128 << key_code as usize);
    }

    pub fn release_all(&mut self) {
        self.bitfield = 0;
    }
}

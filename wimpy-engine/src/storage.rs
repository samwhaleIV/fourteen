#[derive(Default)]
pub struct KeyValueStore {
    //todo... dictionary structure
}

enum StorageValueType {
    String,
    Integer,
    Flag,
}

pub trait KeyValueStoreIO {
    fn export<'a>(&self) -> &'a [u8];
    fn import(&mut self,data: &[u8]);
}

impl KeyValueStoreIO for KeyValueStore {
    fn export<'a>(&self) -> &'a [u8] {
        todo!();
    }
    fn import(&mut self,data: &[u8]) {
        self.delete_all();
        todo!();
    }
}

impl KeyValueStore {
    fn delete_all(&mut self) {
        todo!()
    }

    fn set_string(&mut self,key: &'static str,value: &'static str) {
        todo!()
    }

    fn set_u32(&mut self,key: &'static str,value: u32) {
        todo!()
    }

    fn set_flag(&mut self,key: &'static str) {
        todo!()
    }

    fn delete(&mut self,key: &'static str) {
        todo!()
    }

    fn get_string(&self,key: &'static str) -> Option<&'static str> {
        todo!()
    }

    fn get_u32(&self,key: &'static str) -> Option<u32> {
        todo!()
    }

    fn has_flag(&self,key: &'static str) -> bool {
        todo!()
    }
}

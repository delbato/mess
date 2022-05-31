use std::{any::Any, sync::{Arc, Mutex}, ops::Deref};

use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{value::Value, prelude::{Type}};

pub struct Adapter {
    adapter_impl: Box<dyn AdapterImpl>
}

impl Adapter {
    pub fn new<A: AdapterImpl + 'static>(adapter: A) -> Self {
        Self { adapter_impl: Box::new(adapter) }
    }

    pub fn get_arg<T: DeserializeOwned>(&self, arg_index: usize) -> T {
        let arg_bytes = self.adapter_impl.get_arg_bytes(arg_index);
        bincode::deserialize(&arg_bytes).unwrap()
    }

    pub fn get_foreign_arg<T: 'static>(&self, arg_index: usize) -> Arc<Mutex<T>> {
        let arg_bytes = self.adapter_impl.get_arg_bytes(arg_index);
        let ptr: u64 = bincode::deserialize(&arg_bytes).unwrap();
        self.get_foreign_ptr(ptr)
    }

    pub fn get_foreign_ptr<T: 'static>(&self, ptr: u64) -> Arc<Mutex<T>> {
        let raw_box = self.adapter_impl.get_foreign_ptr(ptr);
        let arc_box: Box<Arc<Mutex<T>>> = raw_box.downcast().expect("ERROR WHEN DOWNCASTING");
        *arc_box
    }

    pub fn insert_foreign_object<T: 'static>(&mut self, val: T) -> u64 {
        let val_arc = Arc::new(Mutex::new(val));
        let any_box: Box<dyn Any> = Box::new(val_arc);
        self.adapter_impl.insert_foreign_ptr(any_box)
    }

    pub fn ret_foreign_object<T: 'static>(&mut self, val: T) {
        let ptr = self.insert_foreign_object(val);
        self.ret(ptr);
    }

    pub fn ret<T: Serialize>(&mut self, val: T) {
        let bytes = bincode::serialize(&val).unwrap();
        self.adapter_impl.ret(&bytes);
    }
}

pub trait AdapterImpl {
    fn ret(&mut self, bytes: &[u8]);
    fn get_arg_bytes(&self, arg_index: usize) -> Vec<u8>;
    fn get_foreign_ptr(&self, ptr: u64) -> Box<dyn Any>;
    fn insert_foreign_ptr(&mut self, object: Box<dyn Any>) -> u64;
}
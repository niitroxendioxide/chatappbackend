use std::sync::{atomic::AtomicUsize, OnceLock, Mutex};
use serde::Serialize;

/*
* Contadores estáticos
* Son "clases" (acá se llama structs, por que no es en base a objetos, sino a estructuras de datos)
* En el caso de AtomicUsize es un integer de tamaño indefinido y unsigned (sin positivo-negativo, solo natural) & es un contador atómico
* o sea que no puede ser interrumpido y va a ser correcto
* En el caso de messages necesitamos un lock que pertenece a un mutex (vimos esto en vigilante) y que se mutex controle un Vector
* de estructura de datos "UserMessage"
*/

#[derive(Debug, Clone, Serialize)]
pub struct UserMessage {
    pub key: usize,
    pub user: String,
    pub content: String,
    pub timestamp: String,
    pub replying_to: usize,
}

pub static MESSAGE_COUNTER: AtomicUsize = AtomicUsize::new(0);
pub static MESSAGES: OnceLock<Mutex<Vec<UserMessage>>> = OnceLock::new();
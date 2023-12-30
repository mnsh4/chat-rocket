#[macro_use]
extern crate rocket;

#[cfg(test)]
mod tests;

use rocket::form::Form;
use rocket::fs::{relative, FileServer};
use rocket::response::stream::{Event, EventStream};
use rocket::serde::{Deserialize, Serialize};
use rocket::tokio::select;
use rocket::tokio::sync::broadcast::{channel, error::RecvError, Sender};
use rocket::{Shutdown, State};

#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, UriDisplayQuery))]
#[serde(crate = "rocket::serde")]
struct Message {
    #[field(validate=len(..30))]
    pub room: String,
    #[field(validate=len(..30))]
    pub username: String,
    pub message: String,
}

// Devuelve un flujo infinito de eventos enviados por el servidor
// Cada evento es un mensaje extraído de una cola de transmisión enviada por el controlador "post"
#[get("/events")]
async fn events(queue: &State<Sender<Message>>, mut end: Shutdown) -> EventStream![] {
    // Se suscribe al canal de mensajes (queue) para recibir mensajes
    let mut rx = queue.subscribe();

    // EventStream! { ... } define un stream de eventos
    EventStream! {
        // Loop infinito que maneja eventos
        loop {
            // Elige entre dos opciones: recibir un mensaje del canal o recibir una señal de finalización
            let msg = select! {
                msg = rx.recv() => match msg {
                    Ok(msg) => msg, // Si se recibe un mensaje correctamente, se procesa
                    Err(RecvError::Closed) => break, // Si el canal se cierra, se termina el loop
                    Err(RecvError::Lagged(_)) => continue, // Si hay retraso, se continua el loop
                },
                _ = &mut end => break, // Si se recibe la señal de finalización, se termina el loop
            };

            // Se emite el evento como un JSON
            yield Event::json(&msg);
        }
    }
}

// Recibir un mensaje del envío de un formulario y transmitirlo a cualquier receptor
#[post("/message", data = "<form>")]
fn post(form: Form<Message>, queue: &State<Sender<Message>>) {
    // Obtiene los datos del formulario y los convierte en la estructura Message
    let _res = queue.send(form.into_inner());
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(channel::<Message>(1024).0)
        .mount("/", routes![post, events])
        .mount("/", FileServer::from(relative!("static")))
}

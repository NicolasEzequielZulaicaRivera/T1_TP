use gtk::prelude::LabelExt;
use gtk::{
    glib,
    prelude::{BuilderExtManual, ButtonExt, ContainerExt, WidgetExt},
    Box, Builder, Button, Label, ListBox, ListBoxRow, Orientation,
};
use packets::{connack::Connack, qos::QoSLevel, unsuback::Unsuback};
use packets::{puback::Puback, publish::Publish, suback::Suback};

use crate::{
    client::ClientError,
    observer::{Message, Observer},
};

use super::{
    subscription_list::SubscriptionList,
    utils::{alert, Icon, InterfaceUtils},
};

/// Observer for the internal client. It sends all messages through
/// a channel to the main GTK thread.
#[derive(Clone)]
pub struct ClientObserver {
    sender: glib::Sender<Message>,
}

impl Observer for ClientObserver {
    fn update(&self, message: Message) {
        if let Err(e) = self.sender.send(message) {
            alert(&format!("Error interno: {}", e));
        }
    }
}

impl ClientObserver {
    /// Creates a new ClientObserver with the given Builder
    /// of the interface
    pub fn new(builder: Builder, subs: SubscriptionList) -> ClientObserver {
        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let mut internal = InternalObserver::new(builder, subs);
        receiver.attach(None, move |message: Message| {
            internal.message_receiver(message);
            glib::Continue(true)
        });

        ClientObserver { sender }
    }
}

/// Internal structure for the ClientObserver, which stores
/// the interface's Builder and runs in the main GKT thread
struct InternalObserver {
    builder: Builder,
    subs: SubscriptionList,
}

impl InterfaceUtils for InternalObserver {
    fn builder(&self) -> &Builder {
        &self.builder
    }
}

impl InternalObserver {
    /// Creates a new InternalObserver with the given
    /// interface builder
    fn new(builder: Builder, subs: SubscriptionList) -> Self {
        Self { builder, subs }
    }

    /// Receives a message and updates the interface
    /// accordingly
    fn message_receiver(&mut self, message: Message) {
        match message {
            Message::Publish(publish) => {
                self.add_publish(publish);
            }
            Message::Connected(result) => {
                self.connected(result);
            }
            Message::Published(result) => {
                self.published(result);
            }
            Message::Subscribed(result) => {
                self.subscribed(result);
            }
            Message::Unsubscribed(result) => {
                self.unsubscribed(result);
            }
            Message::InternalError(error) => {
                alert(&format!(
                    "Error interno: {}\n\nSe recomienda reiniciar el cliente",
                    error
                ));
                let dis: Button = self.builder().object("discon_btn").unwrap();
                dis.clicked();
            }
        }
    }

    /// Re-enables the interface and shows information
    /// about the result of the subscribe operation
    fn subscribed(&mut self, result: Result<Suback, ClientError>) {
        self.sensitive(true);
        match result {
            Ok(suback) => {
                self.icon(Icon::Ok);
                self.status_message("Suscrito");
                self.subs.add_subs(suback.topics());
            }
            Err(error) => {
                self.icon(Icon::Error);
                self.status_message(&format!("No se pudo suscribir: {}", error));
            }
        }
    }

    /// Re-enables the interface and shows information
    /// about the result of the publish operation
    fn published(&mut self, result: Result<Option<Puback>, ClientError>) {
        self.sensitive(true);
        if let Err(e) = result {
            self.icon(Icon::Error);
            self.status_message(&format!("No se pudo publicar: {}", e));
        } else {
            self.icon(Icon::Ok);
            self.status_message("Publicado");
        }
    }

    /// Adds a new received publish packet to the feed
    fn add_publish(&mut self, publish: Publish) {
        let list: ListBox = self.builder.object("sub_msgs").unwrap();

        let row = ListBoxRow::new();
        row.add(&get_box(
            publish.topic_name(),
            publish.payload().unwrap_or(&"".to_string()),
            publish.qos(),
        ));

        list.add(&row);
        list.show_all();
    }

    /// Re-enables the interface and shows information
    /// about the result of the connect operation. If
    /// it succeed it switches to the connected/content menu
    fn connected(&mut self, result: Result<Connack, ClientError>) {
        if let Err(e) = result {
            self.connection_info(None);
            self.sensitive(true);
            self.icon(Icon::Error);
            self.status_message(&format!("No se pudo conectar: {}", e));
        } else {
            self.show_content_menu();
            self.icon(Icon::Ok);
            self.status_message("Conectado");
        }
    }

    /// Re-enables the interfaces and shows information
    /// about the result of the unsubscribe operation
    fn unsubscribed(&mut self, result: Result<Unsuback, ClientError>) {
        self.sensitive(true);
        match result {
            Ok(unsuback) => {
                self.icon(Icon::Ok);
                self.status_message("Desuscrito");
                self.subs.remove_subs(unsuback.topics());
            }
            Err(error) => {
                self.icon(Icon::Error);
                self.status_message(&format!("No se pudo desuscribir: {}", error));
            }
        }
    }
}

#[doc(hidden)]
fn get_box(topic: &str, payload: &str, qos: QoSLevel) -> Box {
    let outer_box = Box::new(Orientation::Vertical, 5);
    let inner_box = Box::new(Orientation::Horizontal, 5);
    let label_topic: Label = Label::new(Some(&("• ".to_owned() + topic)));
    let label_qos: Label = Label::new(Some(&format!("- QoS: {}", qos as u8)));
    let label_payload: Label = Label::new(Some(payload));
    label_topic.set_line_wrap(true);
    label_qos.set_line_wrap(true);
    label_payload.set_line_wrap(true);
    inner_box.add(&label_topic);
    inner_box.add(&label_qos);
    outer_box.add(&inner_box);
    outer_box.add(&label_payload);
    outer_box
}

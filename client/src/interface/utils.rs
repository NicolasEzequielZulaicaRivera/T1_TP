use gtk::{
    prelude::{BuilderExtManual, DialogExt, LabelExt, StackExt, WidgetExt},
    Box, Builder, Button, ButtonsType, DialogFlags, Label, MessageDialog, MessageType, Stack,
    Window,
};

/// Icons of the status bar
pub(crate) enum Icon {
    Ok,
    Error,
    Loading,
}

impl From<Icon> for &str {
    fn from(icon: Icon) -> Self {
        match icon {
            Icon::Ok => "ok",
            Icon::Error => "error",
            Icon::Loading => "loading",
        }
    }
}

pub(crate) trait InterfaceUtils {
    /// Gets the interface builder. Used to give access
    /// the different utilities of this trait access to
    /// the interface
    fn builder(&self) -> &Builder;

    /// Changes the status bar icon to the given one
    fn icon(&self, icon: Icon) {
        let status_icon: Stack = self.builder().object("status_icon").unwrap();
        status_icon.set_visible_child_name(<&str>::from(icon));
    }

    /// Switches the interface to the connect menu
    fn show_connect_menu(&self) {
        let stack: Stack = self.builder().object("content").unwrap();
        stack.set_visible_child_name("box_connection");
        self.sensitive(true);
    }

    /// Switches the interface to the connected/content menu
    fn show_content_menu(&self) {
        let stack: Stack = self.builder().object("content").unwrap();
        stack.set_visible_child_name("box_connected");
        self.sensitive(true);
    }

    /// Allows or disallows the user to touch or write anything
    fn sensitive(&self, sensitive: bool) {
        let content: Stack = self.builder().object("content").unwrap();
        let disconnect_button: Button = self.builder().object("discon_btn").unwrap();
        content.set_sensitive(sensitive);
        disconnect_button.set_sensitive(sensitive);
    }

    /// Sets the status bar message
    fn status_message(&self, msg: &str) {
        let status_text: Label = self.builder().object("status_label").unwrap();
        status_text.set_text(msg);
    }

    /// If msg is Some(_), it sets and shows the connection
    /// info bar with the given message. If it is None,
    /// it hides it.
    fn connection_info(&self, msg: Option<&str>) {
        let info_box: Box = self.builder().object("info_box").unwrap();
        if let Some(text) = msg {
            let label: Label = self.builder().object("connection_info").unwrap();
            info_box.set_visible(true);
            label.set_text(text);
        } else {
            info_box.set_visible(false);
        }
    }
}

/// Creates an error popup with the given message that
/// blocks the current thread until the user closes it
pub(crate) fn alert(message: &str) {
    let dialog = MessageDialog::new(
        None::<&Window>,
        DialogFlags::MODAL,
        MessageType::Error,
        ButtonsType::Close,
        message,
    );
    dialog.run();
    dialog.emit_close();
}
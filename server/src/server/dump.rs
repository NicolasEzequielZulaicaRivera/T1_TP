use std::{
    fs::{self},
    io::{self},
    net::{SocketAddr, TcpStream},
    path::MAIN_SEPARATOR,
    sync::{Arc, Mutex, RwLock},
};

use serde_json::json;
use threadpool::ThreadPool;
use tracing::debug;

use crate::{clients_manager::ClientsManager, topic_handler::TopicHandler, Config, Server};

use super::{server_error::ServerErrorKind, ServerError, ServerResult};

impl<C: Config> Server<C> {
    pub fn try_restore(config: &C, threadpool_size: usize) -> ServerResult<Option<Arc<Server<C>>>> {
        let dump_path = match config.dump_info() {
            Some(dump_info) => dump_info.0,
            None => return Ok(None),
        };

        let json_str = match fs::read_to_string(dump_path) {
            Ok(json_str) => json_str,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(err) => return Err(ServerError::from(err)),
        };

        let (topic_handler, mut clients_manager) = Server::<C>::restore_from_json(&json_str)?;
        let shutdown_info = clients_manager.get_mut()?.shutdown(false)?;
        clients_manager.get_mut()?.set_auth(config.authenticator());
        for client_id in shutdown_info.clean_session_ids {
            topic_handler.remove_client(&client_id)?;
        }

        let server = Server {
            clients_manager,
            config: config.clone(),
            topic_handler,
            pool: Mutex::new(ThreadPool::new(threadpool_size)),
        };
        let server = Arc::new(server);
        for (id, last_will) in shutdown_info.last_will_packets {
            server.send_last_will(last_will, &id)?;
        }
        Ok(Some(server))
    }

    fn restore_from_json(
        json_str: &str,
    ) -> ServerResult<(TopicHandler, RwLock<ClientsManager<TcpStream, SocketAddr>>)> {
        let json: serde_json::Value = match serde_json::from_str(json_str) {
            Ok(json) => json,
            Err(err) => {
                return Err(ServerError::new_kind(
                    &err.to_string(),
                    ServerErrorKind::DumpError,
                ))
            }
        };

        if let serde_json::Value::Object(mut obj) = json {
            let topic_handler = obj.remove("topic_handler").unwrap();
            let clients_manager = obj.remove("clients_manager").unwrap();
            Ok((
                serde_json::from_value(topic_handler).map_err(|err| {
                    ServerError::new_kind(&err.to_string(), ServerErrorKind::DumpError)
                })?,
                serde_json::from_value(clients_manager).map_err(|err| {
                    ServerError::new_kind(&err.to_string(), ServerErrorKind::DumpError)
                })?,
            ))
        } else {
            panic!("Invalid json");
        }
    }

    pub fn dump(&self) -> ServerResult<()> {
        if let Some(dump_info) = self.config.dump_info() {
            debug!("DUMP");
            let topic_handler = serde_json::to_value(&self.topic_handler).map_err(|err| {
                ServerError::new_kind(&err.to_string(), ServerErrorKind::DumpError)
            })?;
            let clients_manager = serde_json::to_value(&self.clients_manager).map_err(|err| {
                ServerError::new_kind(&err.to_string(), ServerErrorKind::DumpError)
            })?;
            let json = json!({
                "topic_handler": topic_handler,
                "clients_manager": clients_manager
            });

            if let Some((folder, _)) = dump_info.0.rsplit_once(MAIN_SEPARATOR) {
                fs::create_dir_all(folder)?;
            }
            fs::write(dump_info.0, serde_json::to_string_pretty(&json)?)?;
        }
        Ok(())
    }
}

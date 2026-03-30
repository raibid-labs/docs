use super::{ClientId, SessionManager};
use anyhow::Result;
use scarab_protocol::{ControlMessage, SessionInfo, SessionResponse};
use std::sync::Arc;

/// Handle session-related control messages
pub async fn handle_session_command(
    msg: ControlMessage,
    session_manager: &Arc<SessionManager>,
    client_id: ClientId,
) -> Result<Option<SessionResponse>> {
    match msg {
        ControlMessage::SessionCreate { name } => {
            log::info!("Client {} creating session: {}", client_id, name);

            match session_manager.create_session(name.to_string(), 80, 24) {
                Ok(id) => {
                    let session = session_manager.get_session(&id).unwrap();
                    Ok(Some(SessionResponse::Created {
                        id: id.to_string(),
                        name: session.name.clone(),
                    }))
                }
                Err(e) => Ok(Some(SessionResponse::Error {
                    message: format!("Failed to create session: {}", e),
                })),
            }
        }

        ControlMessage::SessionDelete { id } => {
            log::info!("Client {} deleting session: {}", client_id, id);

            match session_manager.delete_session(&id.to_string()) {
                Ok(_) => Ok(Some(SessionResponse::Deleted { id: id.clone() })),
                Err(e) => Ok(Some(SessionResponse::Error {
                    message: format!("Failed to delete session: {}", e),
                })),
            }
        }

        ControlMessage::SessionList => {
            log::info!("Client {} listing sessions", client_id);

            let sessions = session_manager.list_sessions();
            let session_infos: Vec<SessionInfo> = sessions
                .into_iter()
                .map(
                    |(id, name, created_at, last_attached, attached_clients)| SessionInfo {
                        id,
                        name,
                        created_at,
                        last_attached,
                        attached_clients: attached_clients as u32,
                    },
                )
                .collect();

            Ok(Some(SessionResponse::List {
                sessions: session_infos,
            }))
        }

        ControlMessage::SessionAttach { id } => {
            log::info!("Client {} attaching to session: {}", client_id, id);

            match session_manager.attach_client(&id.to_string(), client_id) {
                Ok(_) => Ok(Some(SessionResponse::Attached { id: id.clone() })),
                Err(e) => Ok(Some(SessionResponse::Error {
                    message: format!("Failed to attach to session: {}", e),
                })),
            }
        }

        ControlMessage::SessionDetach { id } => {
            log::info!("Client {} detaching from session: {}", client_id, id);

            match session_manager.detach_client(&id.to_string(), client_id) {
                Ok(_) => Ok(Some(SessionResponse::Detached { id: id.clone() })),
                Err(e) => Ok(Some(SessionResponse::Error {
                    message: format!("Failed to detach from session: {}", e),
                })),
            }
        }

        ControlMessage::SessionRename { id, new_name } => {
            log::info!(
                "Client {} renaming session {} to {}",
                client_id,
                id,
                new_name
            );

            match session_manager.rename_session(&id.to_string(), new_name.to_string()) {
                Ok(_) => Ok(Some(SessionResponse::Renamed {
                    id: id.clone(),
                    new_name: new_name.clone(),
                })),
                Err(e) => Ok(Some(SessionResponse::Error {
                    message: format!("Failed to rename session: {}", e),
                })),
            }
        }

        _ => {
            // Not a session command
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_session_commands() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let manager = Arc::new(SessionManager::new(db_path).unwrap());

        // Create session
        let response = handle_session_command(
            ControlMessage::SessionCreate {
                name: "test".to_string().into(),
            },
            &manager,
            1,
        )
        .await
        .unwrap();

        assert!(matches!(response, Some(SessionResponse::Created { .. })));

        // List sessions
        let response = handle_session_command(ControlMessage::SessionList, &manager, 1)
            .await
            .unwrap();

        if let Some(SessionResponse::List { sessions }) = response {
            assert_eq!(sessions.len(), 1);
        } else {
            panic!("Expected List response");
        }
    }
}

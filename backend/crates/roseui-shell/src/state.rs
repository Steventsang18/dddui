use std::sync::Arc;

use roseui_system::ClientPrefService;

use crate::stt::SttService;
use crate::traits::ShellServiceRef;

#[derive(Clone)]
pub struct ShellRouterState {
    pub shell_service: ShellServiceRef,
    pub stt_service: Arc<SttService>,
    pub client_pref_service: ClientPrefService,
}

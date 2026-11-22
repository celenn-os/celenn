// The application in front in a child's session, for the "Most used
// applications" of the parent's window: the session's agent asks COSMIC for
// its windows (the toplevel info protocol, as the dock and the workspaces
// view do) and keeps the app id of the active one. The agent writes it to
// parental_rules::FOCUS every few seconds; the service adds the screen time
// it counts to that application (parental_daemon.rs). Only the app id is
// kept: never the window titles.

use cosmic::cctk::sctk::{
    self,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
};
use cosmic::cctk::toplevel_info::{ToplevelInfoHandler, ToplevelInfoState};
use cosmic::cctk::wayland_client::{
    Connection, QueueHandle, globals::registry_queue_init, protocol::wl_output,
};
use cosmic::cctk::wayland_protocols::ext::foreign_toplevel_list::v1::client::ext_foreign_toplevel_handle_v1::ExtForeignToplevelHandleV1;
use cosmic::cctk::cosmic_protocols::toplevel_info::v1::client::zcosmic_toplevel_handle_v1::State;
use std::sync::{Arc, Mutex};

/// The app id of the active window, shared with the agent's loop
pub type Focus = Arc<Mutex<Option<String>>>;

struct Watcher {
    registry_state: RegistryState,
    output_state: OutputState,
    toplevels: ToplevelInfoState,
    focus: Focus,
}

impl Watcher {
    fn changed(&mut self) {
        let active = self
            .toplevels
            .toplevels()
            .find(|info| info.state.contains(&State::Activated))
            .map(|info| info.app_id.clone());
        if let Ok(mut focus) = self.focus.lock() {
            *focus = active;
        }
    }
}

/// Follows the active window until the session ends; returns at once when
/// the compositor does not tell (another desktop)
pub fn watch(focus: Focus) {
    let Ok(conn) = Connection::connect_to_env() else {
        return;
    };
    let Ok((globals, mut queue)) = registry_queue_init(&conn) else {
        return;
    };
    let qh = queue.handle();
    let registry_state = RegistryState::new(&globals);
    let Some(toplevels) = ToplevelInfoState::try_new(&registry_state, &qh) else {
        eprintln!("No window information from the compositor: no application times");
        return;
    };
    let mut watcher = Watcher {
        output_state: OutputState::new(&globals, &qh),
        registry_state,
        toplevels,
        focus,
    };
    while queue.blocking_dispatch(&mut watcher).is_ok() {}
}

impl ProvidesRegistryState for Watcher {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    sctk::registry_handlers!(OutputState);
}

// The toplevel info needs the outputs bound
impl OutputHandler for Watcher {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}

    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}

    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}

impl ToplevelInfoHandler for Watcher {
    fn toplevel_info_state(&mut self) -> &mut ToplevelInfoState {
        &mut self.toplevels
    }

    fn new_toplevel(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &ExtForeignToplevelHandleV1,
    ) {
        self.changed();
    }

    fn update_toplevel(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &ExtForeignToplevelHandleV1,
    ) {
        self.changed();
    }

    fn toplevel_closed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &ExtForeignToplevelHandleV1,
    ) {
        self.changed();
    }

    fn info_done(&mut self, _: &Connection, _: &QueueHandle<Self>) {
        self.changed();
    }
}

sctk::delegate_output!(Watcher);
sctk::delegate_registry!(Watcher);
cosmic::cctk::delegate_toplevel_info!(Watcher);

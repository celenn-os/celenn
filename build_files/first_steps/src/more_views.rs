// The pages of Tools' tools (more.rs): Inside, Battery, NVIDIA, Advanced,
// the apps lists, Look and Repair. more.rs keeps their state, the jobs they
// run and the rows they share.

use super::*;

impl Page {
    /// What the computer is made of, and how it starts
    pub(super) fn inside<'a>(&'a self, state: &'a State) -> Element<'a, Message> {
        let hw = crate::hardware::hardware();
        // What it is made of: always there, so the page is never empty
        let model = match (&hw.virtual_machine, hw.model.is_empty()) {
            (Some(hypervisor), _) => fill(tr("Virtual machine ({})"), hypervisor),
            (None, false) => hw.model.clone(),
            (None, true) => tr("Unknown").to_owned(),
        };
        let mut processor: Vec<String> = Vec::new();
        if !hw.cpu.is_empty() {
            processor.push(hw.cpu.clone());
        }
        if hw.cores > 0 {
            processor.push(fill(tr("{} cores"), &hw.cores.to_string()));
        }
        if hw.memory_gb > 0 {
            processor.push(fill(tr("{} GB of memory"), &hw.memory_gb.to_string()));
        }
        let processor = if processor.is_empty() {
            tr("Unknown").to_owned()
        } else {
            processor.join(", ")
        };
        let graphics = match (hw.gpus.is_empty(), state.nvidia) {
            (true, _) => tr("Unknown").to_owned(),
            // NVIDIA: the row under it says which driver
            (false, Some(_)) => hw.gpus.join(", "),
            (false, None) => fill(
                tr("{}. The open driver built into Linux works with it: nothing to install."),
                &hw.gpus.join(", "),
            ),
        };
        let startup = match hw.boot {
            Boot::Bios => "Legacy BIOS",
            Boot::Uefi(Some(true)) => "UEFI, Secure Boot on",
            Boot::Uefi(Some(false)) => "UEFI, Secure Boot off",
            Boot::Uefi(None) => "UEFI",
        };
        settings::section()
            .add(info_row("computer-symbolic", "Model", model))
            .add(info_row(
                "power-profile-performance-symbolic",
                "Processor and memory",
                processor,
            ))
            .add(info_row(
                "applications-graphics-symbolic",
                "Graphics card",
                graphics,
            ))
            .add(info_row(
                "system-shutdown-symbolic",
                "Startup",
                tr(startup).to_owned(),
            ))
            .into()
    }

    /// The battery charge limit, or why there is nothing to set
    pub(super) fn battery<'a>(&self, state: &State) -> Element<'a, Message> {
        match state.battery_limit {
            Some(on) => self.row(
                "battery-good-symbolic",
                "Battery charge limit",
                "Charging stops at 80 %: the battery lasts longer when the laptop stays plugged in.",
                self.toggler(on, Job::BatteryLimit),
                None,
            ),
            None => self.row(
                "battery-missing-symbolic",
                "Battery charge limit",
                "No battery on this computer: nothing to set.",
                widget::space::horizontal().width(0).into(),
                None,
            ),
        }
    }

    /// The NVIDIA edition and its driver key
    pub(super) fn nvidia<'a>(&'a self, state: &'a State) -> Element<'a, Message> {
        let mut section = settings::section();
        match state.nvidia {
            Some(true) => {
                section = section.add(match state.driver_key {
                    None => self.row(
                        "video-display-symbolic",
                        "NVIDIA graphics card",
                        "You use the NVIDIA edition, with NVIDIA's own driver.",
                        widget::space::horizontal().width(0).into(),
                        None,
                    ),
                    Some(DriverKey::Missing) => self.row(
                        "video-display-symbolic",
                        "NVIDIA graphics card",
                        "Secure Boot does not let NVIDIA's driver load yet. Allow it once: at the next restart, a blue screen asks for the password celennos.",
                        self.button(Job::DriverKey, "Allow", true),
                        None,
                    ),
                    Some(DriverKey::Queued) => self.row(
                        "video-display-symbolic",
                        "NVIDIA graphics card",
                        "At the next restart, a blue screen asks for the password celennos: choose Enroll MOK, Continue, Yes, type celennos, then Reboot. The keyboard may be in QWERTY.",
                        widget::space::horizontal().width(0).into(),
                        None,
                    ),
                });
            }
            Some(false) => {
                section = section.add(self.row(
                    "video-display-symbolic",
                    "NVIDIA graphics card",
                    "Use the NVIDIA edition of Celenn OS, with NVIDIA's own driver, for games and graphics apps.",
                    self.button(Job::Nvidia, "Use it", true),
                    Some(Job::Nvidia),
                ));
            }
            None => {}
        }
        if state.nvidia.is_none() {
            section = section.add(self.row(
                "video-display-symbolic",
                "NVIDIA graphics card",
                "No NVIDIA graphics card in this computer: its open driver is already the right one.",
                widget::space::horizontal().width(0).into(),
                None,
            ));
        }
        section.into()
    }

    /// For experienced users: the firmware settings, virtual machines,
    /// FlatPark
    pub(super) fn advanced<'a>(&'a self, state: &'a State) -> Element<'a, Message> {
        let firmware = if state.efi {
            self.row(
                "preferences-system-symbolic",
                "Firmware settings (BIOS/UEFI)",
                "Restart into the computer's own settings: boot order, virtualization, Secure Boot…",
                self.button(Job::Firmware, "Restart", false),
                Some(Job::Firmware),
            )
        } else {
            self.row(
                "preferences-system-symbolic",
                "Firmware settings (BIOS/UEFI)",
                "This computer starts in legacy BIOS mode. To open its settings, press the key shown when it powers on, often F2, F10 or Del.",
                widget::space::horizontal().width(0).into(),
                None,
            )
        };
        settings::section()
            .add(firmware)
            .add(self.row(
                "computer-symbolic",
                "Virtual machines",
                "virt-manager, to run Windows or another Linux in a window.",
                self.install(state.vms, Job::Vms),
                None,
            ))
            .add(self.row(
                "system-software-install-symbolic",
                "FlatPark",
                "More apps in COSMIC Store, from outside Flathub. Less checked.",
                self.toggler(state.flatpark, Job::FlatPark),
                None,
            ))
            .into()
    }

    /// The kinds of apps as tiles; or the list of the category opened
    pub(super) fn software<'a>(&'a self, state: &'a State) -> Element<'a, Message> {
        if let Some(n) = self.open {
            return self.list(state, n);
        }
        let tiles = CATEGORIES
            .iter()
            .enumerate()
            .map(|(n, category)| {
                apps::tile(
                    category,
                    apps::tile_caption(category, &BTreeSet::new(), &state.installed[n]),
                    self.applying
                        .is_none()
                        .then_some(Message::ListOpen(Some(n))),
                )
            })
            .collect();
        apps::grid(tiles)
    }

    /// A category's apps by group, each with its checkbox, then Apply
    pub(super) fn list<'a>(&'a self, state: &'a State, n: usize) -> Element<'a, Message> {
        let cosmic_theme::Spacing {
            space_xxs, space_s, ..
        } = theme::spacing();
        let category = &CATEGORIES[n];
        let idle = self.applying.is_none();
        let rows = apps::checklist(
            category,
            &self.chosen[n],
            |_, tool| idle && (state.brew || matches!(tool.kind, ToolKind::Flatpak(_))),
            move |i, on| Message::Tick(n, i, on),
        );
        let section = rows
            .into_iter()
            .fold(settings::section(), |section, row| section.add(row));
        let changed = self.chosen[n] != state.installed[n];
        let apply: Element<'a, Message> = if self.applying == Some(n) {
            widget::row::with_capacity(2)
                .spacing(space_xxs)
                .align_y(Alignment::Center)
                .push(widget::progress_bar::indeterminate_circular().size(20.0))
                .push(text::body(tr("Working…")))
                .into()
        } else {
            widget::button::suggested(tr("Apply"))
                .on_press_maybe(
                    (changed && self.busy.is_none() && idle).then_some(Message::ListApply(n)),
                )
                .into()
        };
        let mut notes = vec![tr(apps::TICK_NOTE)];
        if n == DEVELOPMENT && !state.brew {
            notes.push(tr(apps::NO_BREW));
        } else if !category.note.is_empty() {
            notes.push(tr(category.note));
        }
        let note = notes.join(" ");
        widget::column::with_capacity(3)
            .spacing(space_s)
            .push(text::heading(tr(category.title)))
            .push(section)
            .push(
                text::caption(note)
                    .wrapping(cosmic::iced::core::text::Wrapping::Word)
                    .width(Length::Fill),
            )
            .push(
                widget::row::with_capacity(3)
                    .spacing(space_s)
                    .align_y(Alignment::Center)
                    .push(
                        widget::button::standard(tr("Back to the list"))
                            .on_press_maybe(idle.then_some(Message::ListOpen(None))),
                    )
                    .push(widget::space::horizontal().width(Length::Fill))
                    .push(apply),
            )
            .into()
    }

    /// Dark or light, the layout, the dock, more in Settings
    pub(super) fn look(&self, state: &State) -> Element<'_, Message> {
        let space_s = theme::spacing().space_s;
        let layout_buttons = [(false, "Top bar and dock"), (true, "Bar at the bottom")]
            .into_iter()
            .fold(
                widget::row::with_capacity(2).spacing(space_s),
                |row, (bottom, label)| {
                    let button = if state.bottom_bar == bottom {
                        widget::button::suggested(tr(label))
                    } else {
                        widget::button::standard(tr(label))
                    };
                    row.push(button.on_press(Message::Layout(bottom)))
                },
            );
        let mut look = settings::section()
            .add(self.row(
                "weather-clear-night-symbolic",
                "Dark theme",
                "Light text on dark backgrounds. Off: the light theme.",
                self.toggler(state.dark, Job::Dark),
                None,
            ))
            .add(self.row(
                "video-display-symbolic",
                "Layout",
                "Top bar and dock: like the Mac. One bar at the bottom: like Windows.",
                layout_buttons.into(),
                None,
            ));
        // The dock only exists with the top bar
        if !state.bottom_bar {
            look = look.add(
                self.row(
                    "view-app-grid-symbolic",
                    "Dock always visible",
                    "Off: it hides when a window covers it.",
                    widget::toggler(state.dock_visible)
                        .on_toggle(Message::DockVisible)
                        .into(),
                    None,
                ),
            );
        }
        let look = look.add(
            self.row(
                "preferences-desktop-appearance-symbolic",
                "More in Settings",
                "Accent color, wallpaper, panel, dock, fonts…",
                widget::button::standard(tr("Open"))
                    .on_press(Message::OpenSettings)
                    .into(),
                None,
            ),
        );
        look.into()
    }

    /// The Celenn OS look back, or every COSMIC setting
    pub(super) fn repair(&self) -> Element<'_, Message> {
        settings::section()
            .add(self.row(
                "edit-undo-symbolic",
                "The Celenn OS look",
                "Back to the Celenn OS colors, dark theme, icons, fonts and wallpaper. Panel, dock, keyboard and shortcuts stay.",
                self.button(Job::ResetTheme, "Restore", false),
                None,
            ))
            .add(self.row(
                "document-revert-symbolic",
                "All COSMIC settings",
                "Back to the Celenn OS defaults, for every setting. A copy of the current ones is kept.",
                self.button(Job::ResetConfig, "Reset", false),
                Some(Job::ResetConfig),
            ))
            .into()
    }
}

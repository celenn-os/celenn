#!/usr/bin/python3
"""Patches COSMIC Store for Celenn OS: FlatPark as a recommended source.

COSMIC Store's "Software repositories" panel lists two built-in recommended
Flatpak sources (Flathub, COSMIC Flatpak), each with an Add or Remove button,
for the user's own Flatpak installation. Celenn OS offers FlatPark but
leaves it off (its apps are not reviewed like Flathub's), so it becomes a
third recommended source, off until the user adds it:

1. The FlatPark entry reads its descriptor from the image
   (/usr/share/celenn/flatpak/flatpark.flatpakrepo, from
   projectbluefin/common), so it follows common's updates. Its Filter= key
   hides every app that is also on Flathub. It requires Flathub: adding it
   adds Flathub if missing, removing Flathub removes it too. No entry if the
   file is missing.
2. FlatPark is added with priority 0, below Flathub and COSMIC (1).

`ujust toggle-flatpark` turns on or off the same remote.

It also translates the texts that COSMIC Store writes in English in its code
(each marked "TODO: translate" upstream), with English, French, German,
Spanish, Italian and Brazilian Portuguese texts added to its Fluent files
(other languages fall back to English; to add one, add an append() of
i18n/<language>/cosmic_store.ftl with the same keys):

3. The "System Upgrade" row of the Updates page (rpm-ostree backend), with
   the Celenn OS updates icon (system-software-update) instead of a
   generic package, and the "System" source name.
4. The progress, done and error texts of installs, removals, updates and
   repository changes ("Installing X from Flathub (40%)...").
Each replacement must match exactly once, so the build fails loudly when a
new upstream version changes this code. Usage: patch.py <source dir>
"""

import pathlib
import sys

src = pathlib.Path(sys.argv[1])


def replace(rel, old, new):
    path = src / rel
    text = path.read_text()
    count = text.count(old)
    if count != 1:
        sys.exit(f"{rel}: expected 1 match, found {count}:\n{old}")
    path.write_text(text.replace(old, new))


replace(
    "src/main.rs",
    """                //TODO: can this be defined in flatpakrepo file?
                requires: vec!["flathub".to_string()],
            });
""",
    """                //TODO: can this be defined in flatpakrepo file?
                requires: vec!["flathub".to_string()],
            });
            // Celenn OS: FlatPark, offered but off by default
            static FLATPARK: std::sync::OnceLock<Option<Vec<u8>>> = std::sync::OnceLock::new();
            if let Some(data) = FLATPARK.get_or_init(|| {
                std::fs::read("/usr/share/celenn/flatpak/flatpark.flatpakrepo").ok()
            }) {
                sources.push(Source {
                    backend_name: BackendName::FlatpakUser,
                    id: "flatpark".to_string(),
                    name: "FlatPark".to_string(),
                    kind: SourceKind::Recommended {
                        data: data.as_slice(),
                        enabled: false,
                    },
                    requires: vec!["flathub".to_string()],
                });
            }
""",
)

replace(
    "src/backend/flatpak.rs",
    """                for add in adds.iter() {
                    remotes.push(Remote::from_file(&add.id, &glib::Bytes::from(&add.data))?);
                }
""",
    """                for add in adds.iter() {
                    let remote = Remote::from_file(&add.id, &glib::Bytes::from(&add.data))?;
                    // Celenn OS: FlatPark comes after every other source
                    if add.id == "flatpark" {
                        remote.set_prio(0);
                    }
                    remotes.push(remote);
                }
""",
)


def append(rel, text):
    path = src / rel
    path.write_text(path.read_text().rstrip("\n") + "\n\n" + text)


replace(
    "src/backend/rpm_ostree.rs",
    """        //TODO: translate?
        let source_name = "System";
""",
    """        // Celenn OS: translated
        let source_name = crate::fl!("source-system");
""",
)

replace(
    "src/backend/packagekit.rs",
    """        //TODO: translate?
        let source_name = "System";
""",
    """        // Celenn OS: translated
        let source_name = crate::fl!("source-system");
""",
)

replace(
    "src/backend/rpm_ostree.rs",
    """            icon: widget::icon::from_name("package-x-generic")
                .size(128)
                .handle(),
            info: Arc::new(AppInfo {
                source_id: "rpm-ostree".to_string(),
                source_name: "rpm-ostree".to_string(),
                name: "System Upgrade".to_string(),
                summary: format!("Update to version {}", version),
                description: "An OS update is available via rpm-ostree.".to_string(),
""",
    """            // Celenn OS: translated, with the updates icon
            icon: widget::icon::from_name("system-software-update")
                .size(128)
                .handle(),
            info: Arc::new(AppInfo {
                source_id: "rpm-ostree".to_string(),
                source_name: crate::fl!("source-system"),
                name: crate::fl!("system-upgrade"),
                summary: crate::fl!("system-upgrade-summary", version = version.clone()),
                description: crate::fl!("system-upgrade-description"),
""",
)

# Operation texts: upstream's English, then the translated version
OLD_OPERATION = r"""
impl Operation {
    pub fn pending_text(&self, progress: i32) -> String {
        //TODO: translate
        let verb = match &self.kind {
            OperationKind::Install => "Installing",
            OperationKind::Uninstall { .. } => "Uninstalling",
            OperationKind::Update => "Updating",
            OperationKind::RepositoryAdd(adds) => {
                return format!(
                    "Adding repositories {:?} ({}%)",
                    RepositoryAdd::ids(adds),
                    progress
                );
            }
            OperationKind::RepositoryRemove(rms, _force) => {
                return format!(
                    "Removing repositories {:?} ({}%)",
                    RepositoryRemove::ids(rms),
                    progress
                );
            }
        };
        format!(
            "{} {} from {} ({}%)...",
            verb, self.infos[0].name, self.infos[0].source_name, progress
        )
    }

    pub fn completed_text(&self) -> String {
        //TODO: translate
        let verb = match &self.kind {
            OperationKind::Install => "Installed",
            OperationKind::Uninstall { .. } => "Uninstalled",
            OperationKind::Update => "Updated",
            OperationKind::RepositoryAdd(adds) => {
                return format!("Added repositories {:?}", RepositoryAdd::ids(adds));
            }
            OperationKind::RepositoryRemove(rms, _force) => {
                return format!("Removed repositories {:?}", RepositoryRemove::ids(rms));
            }
        };
        format!(
            "{} {} from {}",
            verb, self.infos[0].name, self.infos[0].source_name
        )
    }

    pub fn failed_dialog(&self, err: &str) -> (String, String) {
        //TODO: translate
        let verb = match &self.kind {
            OperationKind::Install => "install",
            OperationKind::Uninstall { .. } => "uninstall",
            OperationKind::Update => "update",
            OperationKind::RepositoryAdd(adds) => {
                return (
                    "Failed to add repositories".to_string(),
                    format!(
                        "Failed to add repositories {:?}:\n{err}",
                        RepositoryAdd::ids(adds)
                    ),
                );
            }
            OperationKind::RepositoryRemove(rms, _force) => {
                return (
                    "Failed to remove repositories".to_string(),
                    format!(
                        "Failed to remove repositories {:?}:\n{err}",
                        RepositoryRemove::ids(rms)
                    ),
                );
            }
        };
        //TODO: get ids and names from all packages
        (
            format!(
                "Failed to {verb} {} from {}",
                self.infos[0].name, self.infos[0].source_name
            ),
            format!(
                "Failed to {verb} {} ({}) from {} ({}):\n{err}",
                self.infos[0].name,
                self.package_ids[0].raw(),
                self.infos[0].source_name,
                self.infos[0].source_id
            ),
        )
    }
}
"""

NEW_OPERATION = r"""
impl Operation {
    // Celenn OS: translated texts (i18n/*/cosmic_store.ftl, operation-*)
    fn kind_id(&self) -> &'static str {
        match &self.kind {
            OperationKind::Install => "install",
            OperationKind::Uninstall { .. } => "uninstall",
            // The rpm-ostree row is the system itself
            OperationKind::Update
                if self.infos.first().is_some_and(|info| info.source_id == "rpm-ostree") =>
            {
                "update-system"
            }
            OperationKind::Update => "update",
            OperationKind::RepositoryAdd(_) => "add-repositories",
            OperationKind::RepositoryRemove(..) => "remove-repositories",
        }
    }

    fn repository_ids(&self) -> String {
        match &self.kind {
            OperationKind::RepositoryAdd(adds) => RepositoryAdd::ids(adds).join(", "),
            OperationKind::RepositoryRemove(rms, _force) => RepositoryRemove::ids(rms).join(", "),
            _ => String::new(),
        }
    }

    // Repository changes name their repositories, others their first app
    fn is_repository_change(&self) -> bool {
        matches!(
            self.kind,
            OperationKind::RepositoryAdd(_) | OperationKind::RepositoryRemove(..)
        )
    }

    fn name_and_source(&self) -> (String, String) {
        match self.infos.first() {
            Some(info) if !self.is_repository_change() => {
                (info.name.clone(), info.source_name.clone())
            }
            _ => (self.repository_ids(), String::new()),
        }
    }

    pub fn pending_text(&self, progress: i32) -> String {
        let (name, source) = self.name_and_source();
        crate::fl!(
            "operation-pending",
            kind = self.kind_id(),
            name = name,
            source = source,
            percent = progress
        )
    }

    pub fn completed_text(&self) -> String {
        let (name, source) = self.name_and_source();
        crate::fl!(
            "operation-completed",
            kind = self.kind_id(),
            name = name,
            source = source
        )
    }

    pub fn failed_dialog(&self, err: &str) -> (String, String) {
        let (name, source) = self.name_and_source();
        let title = crate::fl!(
            "operation-failed",
            kind = self.kind_id(),
            name = name,
            source = source
        );
        let details = match (self.infos.first(), self.package_ids.first()) {
            (Some(info), Some(id)) if !self.is_repository_change() => format!(
                "{} ({}), {} ({}):",
                info.name,
                id.raw(),
                info.source_name,
                info.source_id
            ),
            _ => format!("{}:", self.repository_ids()),
        };
        let body = format!("{title}\n{details}\n{err}");
        (title, body)
    }
}
"""

replace("src/operation.rs", OLD_OPERATION, NEW_OPERATION)

append("i18n/en/cosmic_store.ftl", """## Celenn OS
source-system = System
system-upgrade = System update
system-upgrade-summary = Update to version { $version }
system-upgrade-description = A new version of the system is available. It is installed in the background and starts at the next restart.
operation-pending = { $kind ->
    [install] Installing { $name } from { $source } ({ $percent }%)...
    [uninstall] Uninstalling { $name } from { $source } ({ $percent }%)...
    [add-repositories] Adding repositories { $name } ({ $percent }%)...
    [remove-repositories] Removing repositories { $name } ({ $percent }%)...
    [update-system] Updating the system ({ $percent }%)...
   *[update] Updating { $name } from { $source } ({ $percent }%)...
}
operation-completed = { $kind ->
    [install] Installed { $name } from { $source }
    [uninstall] Uninstalled { $name } from { $source }
    [add-repositories] Added repositories { $name }
    [remove-repositories] Removed repositories { $name }
    [update-system] System updated: the new version starts at the next restart
   *[update] Updated { $name } from { $source }
}
operation-failed = { $kind ->
    [install] Failed to install { $name } from { $source }
    [uninstall] Failed to uninstall { $name } from { $source }
    [add-repositories] Failed to add repositories { $name }
    [remove-repositories] Failed to remove repositories { $name }
    [update-system] Failed to update the system
   *[update] Failed to update { $name } from { $source }
}
""")

append("i18n/fr/cosmic_store.ftl", """## Celenn OS
source-system = Système
system-upgrade = Mise à jour du système
system-upgrade-summary = Passer à la version { $version }
system-upgrade-description = Une nouvelle version du système est disponible. Elle s'installe en arrière-plan et démarre au prochain redémarrage.
operation-pending = { $kind ->
    [install] Installation de { $name } depuis { $source } ({ $percent } %)...
    [uninstall] Désinstallation de { $name } ({ $source }, { $percent } %)...
    [add-repositories] Ajout des dépôts { $name } ({ $percent } %)...
    [remove-repositories] Suppression des dépôts { $name } ({ $percent } %)...
    [update-system] Mise à jour du système ({ $percent } %)...
   *[update] Mise à jour de { $name } depuis { $source } ({ $percent } %)...
}
operation-completed = { $kind ->
    [install] { $name } installé depuis { $source }
    [uninstall] { $name } désinstallé ({ $source })
    [add-repositories] Dépôts { $name } ajoutés
    [remove-repositories] Dépôts { $name } supprimés
    [update-system] Système mis à jour : la nouvelle version démarre au prochain redémarrage
   *[update] { $name } mis à jour depuis { $source }
}
operation-failed = { $kind ->
    [install] Impossible d'installer { $name } depuis { $source }
    [uninstall] Impossible de désinstaller { $name } ({ $source })
    [add-repositories] Impossible d'ajouter les dépôts { $name }
    [remove-repositories] Impossible de supprimer les dépôts { $name }
    [update-system] Impossible de mettre à jour le système
   *[update] Impossible de mettre à jour { $name } depuis { $source }
}
""")

append("i18n/de/cosmic_store.ftl", """## Celenn OS
source-system = System
system-upgrade = Systemupdate
system-upgrade-summary = Auf Version { $version } aktualisieren
system-upgrade-description = Eine neue Version des Systems ist verfügbar. Sie wird im Hintergrund installiert und beim nächsten Neustart aktiv.
operation-pending = { $kind ->
    [install] { $name } wird aus { $source } installiert ({ $percent } %)...
    [uninstall] { $name } wird entfernt ({ $source }, { $percent } %)...
    [add-repositories] Quellen { $name } werden hinzugefügt ({ $percent } %)...
    [remove-repositories] Quellen { $name } werden entfernt ({ $percent } %)...
    [update-system] Das System wird aktualisiert ({ $percent } %)...
   *[update] { $name } wird aus { $source } aktualisiert ({ $percent } %)...
}
operation-completed = { $kind ->
    [install] { $name } aus { $source } installiert
    [uninstall] { $name } entfernt ({ $source })
    [add-repositories] Quellen { $name } hinzugefügt
    [remove-repositories] Quellen { $name } entfernt
    [update-system] System aktualisiert: Die neue Version startet beim nächsten Neustart
   *[update] { $name } aus { $source } aktualisiert
}
operation-failed = { $kind ->
    [install] { $name } konnte nicht aus { $source } installiert werden
    [uninstall] { $name } konnte nicht entfernt werden ({ $source })
    [add-repositories] Quellen { $name } konnten nicht hinzugefügt werden
    [remove-repositories] Quellen { $name } konnten nicht entfernt werden
    [update-system] Das System konnte nicht aktualisiert werden
   *[update] { $name } konnte nicht aus { $source } aktualisiert werden
}
""")

append("i18n/es/cosmic_store.ftl", """## Celenn OS
source-system = Sistema
system-upgrade = Actualización del sistema
system-upgrade-summary = Actualizar a la versión { $version }
system-upgrade-description = Hay una nueva versión del sistema. Se instala en segundo plano y se inicia en el próximo reinicio.
operation-pending = { $kind ->
    [install] Instalando { $name } desde { $source } ({ $percent } %)...
    [uninstall] Desinstalando { $name } ({ $source }, { $percent } %)...
    [add-repositories] Añadiendo los repositorios { $name } ({ $percent } %)...
    [remove-repositories] Quitando los repositorios { $name } ({ $percent } %)...
    [update-system] Actualizando el sistema ({ $percent } %)...
   *[update] Actualizando { $name } desde { $source } ({ $percent } %)...
}
operation-completed = { $kind ->
    [install] { $name } instalado desde { $source }
    [uninstall] { $name } desinstalado ({ $source })
    [add-repositories] Repositorios { $name } añadidos
    [remove-repositories] Repositorios { $name } quitados
    [update-system] Sistema actualizado: la nueva versión se inicia en el próximo reinicio
   *[update] { $name } actualizado desde { $source }
}
operation-failed = { $kind ->
    [install] No se pudo instalar { $name } desde { $source }
    [uninstall] No se pudo desinstalar { $name } ({ $source })
    [add-repositories] No se pudieron añadir los repositorios { $name }
    [remove-repositories] No se pudieron quitar los repositorios { $name }
    [update-system] No se pudo actualizar el sistema
   *[update] No se pudo actualizar { $name } desde { $source }
}
""")

append("i18n/it/cosmic_store.ftl", """## Celenn OS
source-system = Sistema
system-upgrade = Aggiornamento del sistema
system-upgrade-summary = Aggiorna alla versione { $version }
system-upgrade-description = È disponibile una nuova versione del sistema. Si installa in background e si avvia al prossimo riavvio.
operation-pending = { $kind ->
    [install] Installazione di { $name } da { $source } ({ $percent }%)...
    [uninstall] Disinstallazione di { $name } ({ $source }, { $percent }%)...
    [add-repositories] Aggiunta dei repository { $name } ({ $percent }%)...
    [remove-repositories] Rimozione dei repository { $name } ({ $percent }%)...
    [update-system] Aggiornamento del sistema ({ $percent }%)...
   *[update] Aggiornamento di { $name } da { $source } ({ $percent }%)...
}
operation-completed = { $kind ->
    [install] { $name } installato da { $source }
    [uninstall] { $name } disinstallato ({ $source })
    [add-repositories] Repository { $name } aggiunti
    [remove-repositories] Repository { $name } rimossi
    [update-system] Sistema aggiornato: la nuova versione si avvia al prossimo riavvio
   *[update] { $name } aggiornato da { $source }
}
operation-failed = { $kind ->
    [install] Impossibile installare { $name } da { $source }
    [uninstall] Impossibile disinstallare { $name } ({ $source })
    [add-repositories] Impossibile aggiungere i repository { $name }
    [remove-repositories] Impossibile rimuovere i repository { $name }
    [update-system] Impossibile aggiornare il sistema
   *[update] Impossibile aggiornare { $name } da { $source }
}
""")

append("i18n/pt-BR/cosmic_store.ftl", """## Celenn OS
source-system = Sistema
system-upgrade = Atualização do sistema
system-upgrade-summary = Atualizar para a versão { $version }
system-upgrade-description = Uma nova versão do sistema está disponível. Ela é instalada em segundo plano e começa a valer na próxima reinicialização.
operation-pending = { $kind ->
    [install] Instalando { $name } de { $source } ({ $percent }%)...
    [uninstall] Desinstalando { $name } ({ $source }, { $percent }%)...
    [add-repositories] Adicionando os repositórios { $name } ({ $percent }%)...
    [remove-repositories] Removendo os repositórios { $name } ({ $percent }%)...
    [update-system] Atualizando o sistema ({ $percent }%)...
   *[update] Atualizando { $name } de { $source } ({ $percent }%)...
}
operation-completed = { $kind ->
    [install] { $name } instalado de { $source }
    [uninstall] { $name } desinstalado ({ $source })
    [add-repositories] Repositórios { $name } adicionados
    [remove-repositories] Repositórios { $name } removidos
    [update-system] Sistema atualizado: a nova versão começa a valer na próxima reinicialização
   *[update] { $name } atualizado de { $source }
}
operation-failed = { $kind ->
    [install] Não foi possível instalar { $name } de { $source }
    [uninstall] Não foi possível desinstalar { $name } ({ $source })
    [add-repositories] Não foi possível adicionar os repositórios { $name }
    [remove-repositories] Não foi possível remover os repositórios { $name }
    [update-system] Não foi possível atualizar o sistema
   *[update] Não foi possível atualizar { $name } de { $source }
}
""")

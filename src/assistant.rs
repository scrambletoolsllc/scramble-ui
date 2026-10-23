//! The assistant's models, shared by the Scramble Tools apps: set up in
//! one, they are there in the others, and a change one makes reaches the
//! others while they run. Each model's API key is in the system keychain
//! under [`keychain_service`], the same for every app.
//!
//! The file is `scramble/assistant.toml` in the config folder: under
//! `$XDG_CONFIG_HOME` (`~/.config`) on Linux, `%APPDATA%` on Windows and
//! `~/Library/Application Support` on macOS. Development builds use
//! `assistant-dev.toml` beside it, so they never disturb installed apps.
//! Which model an app uses is the app's own setting.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

/// How often [`changes`] looks at the file.
const POLL: Duration = Duration::from_secs(1);

const HEADER: &str = "\
# The assistant's models, shared by the Scramble Tools apps, such as plaa
# and prev. Their API keys are in the system keychain, not here.

";

/// A model the assistant can use.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Model {
    /// Names the model, and a key kept for it alone in the keychain.
    pub id: String,
    /// `anthropic`, `open-ai`, `gemini`, `ollama` or `open-ai-compatible`.
    pub provider: String,
    pub model: String,
    /// The server's address, for a local or compatible server.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    /// How much a local model reads at once, in tokens, when the app sets
    /// it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<u32>,
    /// The provider's name for the model, when it gives one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Whether the model sees pictures, when the provider said.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vision: Option<bool>,
    /// Settings a newer app added, kept as they are.
    #[serde(flatten)]
    pub other: toml::Table,
}

impl Model {
    /// Whether `other` is the same model on the same server, whatever its
    /// id.
    pub fn same_as(&self, other: &Model) -> bool {
        self.provider == other.provider
            && self.model == other.model
            && self.address == other.address
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Shared {
    #[serde(default)]
    models: Vec<Model>,
    /// What a newer app added, kept as it is.
    #[serde(flatten)]
    other: toml::Table,
}

/// The shared file: the installed apps' when `production`, else the
/// development builds'.
pub fn file(production: bool) -> Option<PathBuf> {
    let name = if production {
        "assistant.toml"
    } else {
        "assistant-dev.toml"
    };
    config_dir().map(|dir| dir.join("scramble").join(name))
}

/// The keychain service the models' keys are under.
pub fn keychain_service(production: bool) -> &'static str {
    if production {
        "scramble-assistant"
    } else {
        "scramble-assistant-dev"
    }
}

#[cfg(not(any(windows, target_os = "macos")))]
fn config_dir() -> Option<PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
}

#[cfg(windows)]
fn config_dir() -> Option<PathBuf> {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
}

#[cfg(target_os = "macos")]
fn config_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join("Library/Application Support"))
}

/// The models in `path`; none when it does not exist yet.
pub fn load(path: &Path) -> Result<Vec<Model>, String> {
    read(path).map(|shared| shared.models)
}

fn read(path: &Path) -> Result<Shared, String> {
    match std::fs::read_to_string(path) {
        Ok(text) => toml::from_str(&text).map_err(|error| error.to_string()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Shared::default()),
        Err(error) => Err(error.to_string()),
    }
}

/// Changes the models in `path` as `edit` says. They are read afresh
/// and the file locked meanwhile, so a change another app makes at the
/// same moment is not lost. Gives the models as saved, and what `edit`
/// gave.
pub fn change<T>(
    path: &Path,
    edit: impl FnOnce(&mut Vec<Model>) -> T,
) -> Result<(Vec<Model>, T), String> {
    let folder = path.parent().ok_or("The file has no folder.")?;
    std::fs::create_dir_all(folder).map_err(|error| error.to_string())?;
    let lock = std::fs::File::create(path.with_extension("lock"))
        .and_then(|lock| lock.lock().map(|()| lock))
        .map_err(|error| error.to_string())?;
    let mut shared = read(path)?;
    let given = edit(&mut shared.models);
    let text = toml::to_string(&shared).map_err(|error| error.to_string())?;
    write(path, &format!("{HEADER}{text}")).map_err(|error| error.to_string())?;
    drop(lock);
    Ok((shared.models, given))
}

/// Writes `path` whole or not at all: to a file beside it first, then
/// put in its place.
fn write(path: &Path, text: &str) -> std::io::Result<()> {
    let mut name = path.file_name().unwrap_or_default().to_owned();
    name.push(".new");
    let new = path.with_file_name(name);
    let mut file = std::fs::File::create(&new)?;
    file.write_all(text.as_bytes())?;
    file.sync_all()?;
    std::fs::rename(&new, path)
}

/// Moves models an app kept in its own settings into the shared file,
/// leaving out those already there. Gives each one's id beside the id it
/// has in the file, which differs where the same model was there already
/// or another had its id.
pub fn adopt(path: &Path, models: Vec<Model>) -> Result<Vec<(String, String)>, String> {
    change(path, |shared| {
        models
            .into_iter()
            .map(|mut model| {
                if let Some(there) = shared.iter().find(|there| there.same_as(&model)) {
                    return (model.id, there.id.clone());
                }
                let id = unused_id(shared, &model.id);
                let old = std::mem::replace(&mut model.id, id.clone());
                shared.push(model);
                (old, id)
            })
            .collect()
    })
    .map(|(_, ids)| ids)
}

/// `base`, or `base` with the first number from 2 that makes it an id
/// no model has.
pub fn unused_id(models: &[Model], base: &str) -> String {
    let taken = |id: &str| models.iter().any(|model| model.id == id);
    if !taken(base) {
        return base.to_owned();
    }
    (2..)
        .map(|number| format!("{base}-{number}"))
        .find(|id| !taken(id))
        .unwrap_or_default()
}

/// The models each time `path` changes, for `Subscription::run_with`;
/// an error where it cannot be read.
pub fn changes(path: PathBuf) -> impl iced::futures::Stream<Item = Result<Vec<Model>, String>> {
    let (mut sender, receiver) = iced::futures::channel::mpsc::channel(1);
    let mut last = stamp(&path);
    std::thread::Builder::new()
        .name("assistant models".into())
        .spawn(move || {
            loop {
                std::thread::sleep(POLL);
                let now = stamp(&path);
                if now == last {
                    continue;
                }
                match sender.try_send(load(&path)) {
                    Ok(()) => last = now,
                    Err(error) if error.is_disconnected() => return,
                    // Full: sent again next time.
                    Err(_) => {}
                }
            }
        })
        .expect("start the assistant models thread");
    receiver
}

/// When the file last changed, and its size; `None` while there is none.
fn stamp(path: &Path) -> Option<(SystemTime, u64)> {
    let metadata = std::fs::metadata(path).ok()?;
    Some((metadata.modified().ok()?, metadata.len()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model(id: &str, model: &str) -> Model {
        Model {
            id: id.to_owned(),
            provider: "ollama".to_owned(),
            model: model.to_owned(),
            address: None,
            context: None,
            name: None,
            vision: None,
            other: toml::Table::new(),
        }
    }

    #[test]
    fn no_file_is_no_models() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(load(&dir.path().join("assistant.toml")).unwrap(), []);
    }

    #[test]
    fn a_change_keeps_what_it_does_not_know() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("scramble").join("assistant.toml");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let text = "later = 1\n\n[[models]]\nid = \"ollama-a\"\nprovider = \"ollama\"\nmodel = \"a\"\ntemperature = 0.5\n";
        std::fs::write(&path, text).unwrap();
        let (models, ()) = change(&path, |models| models.push(model("ollama-b", "b"))).unwrap();
        assert_eq!(models.len(), 2);
        let saved = std::fs::read_to_string(&path).unwrap();
        assert!(saved.starts_with("# The assistant's models"));
        assert!(saved.contains("later = 1"));
        assert!(saved.contains("temperature = 0.5"));
        assert_eq!(load(&path).unwrap(), models);
    }

    #[test]
    fn adopting_leaves_out_what_is_there() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("assistant.toml");
        change(&path, |models| models.push(model("ollama-a", "a"))).unwrap();
        let ids = adopt(
            &path,
            vec![
                model("mine", "a"),
                model("ollama-a", "b"),
                model("ollama-c", "c"),
            ],
        )
        .unwrap();
        let ids: Vec<(&str, &str)> = ids
            .iter()
            .map(|(old, new)| (old.as_str(), new.as_str()))
            .collect();
        assert_eq!(
            ids,
            [
                ("mine", "ollama-a"),
                ("ollama-a", "ollama-a-2"),
                ("ollama-c", "ollama-c")
            ]
        );
        assert_eq!(load(&path).unwrap().len(), 3);
    }

    #[test]
    fn changes_are_heard() {
        use iced::futures::StreamExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("assistant.toml");
        let mut heard = Box::pin(changes(path.clone()));
        change(&path, |models| models.push(model("ollama-a", "a"))).unwrap();
        let models = iced::futures::executor::block_on(heard.next()).unwrap();
        assert_eq!(models.unwrap()[0].id, "ollama-a");
    }
}

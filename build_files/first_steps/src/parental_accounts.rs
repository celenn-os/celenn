// Parental controls: the people's accounts on this computer, as the
// parent's window lists them (parental.rs), and the user name made for a
// new child's account.

use super::*;

#[derive(Clone, Debug)]
pub(super) struct Account {
    pub(super) user: String,
    /// The full name, or the user name
    pub(super) name: String,
    pub(super) admin: bool,
    pub(super) rules: Option<Rules>,
    pub(super) usage: Usage,
    pub(super) extra: Vec<parental_rules::Grant>,
    pub(super) avatar: Option<widget::image::Handle>,
}

/// The people's accounts (user ids 1000 and up, with a login shell):
/// administrators (group wheel) first, then the others
pub(super) fn accounts() -> Vec<Account> {
    let getent = |args: &[&str]| {
        std::process::Command::new("getent")
            .args(args)
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
            .unwrap_or_default()
    };
    let wheel: BTreeSet<String> = getent(&["group", "wheel"])
        .split(':')
        .nth(3)
        .unwrap_or_default()
        .trim()
        .split(',')
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .collect();
    let mut accounts: Vec<Account> = getent(&["passwd"])
        .lines()
        .filter_map(|line| {
            let fields: Vec<&str> = line.split(':').collect();
            let (user, uid, gecos, shell) = (
                fields.first()?,
                fields.get(2)?,
                fields.get(4)?,
                fields.get(6)?,
            );
            let uid: u32 = uid.parse().ok()?;
            if !(1000..60000).contains(&uid)
                || shell.ends_with("nologin")
                || shell.ends_with("false")
            {
                return None;
            }
            let name = gecos.split(',').next().unwrap_or_default().trim();
            let avatar = Path::new(AVATARS).join(user);
            Some(Account {
                user: (*user).to_owned(),
                name: if name.is_empty() {
                    (*user).to_owned()
                } else {
                    name.to_owned()
                },
                admin: wheel.contains(*user),
                rules: Rules::load(user),
                usage: Usage::load(user),
                extra: parental_rules::load_extra(user),
                // No picture of their own: COSMIC's default one (our
                // planet), as on the login screen
                avatar: style::rounded_picture(&avatar, 128)
                    .or_else(|| style::rounded_picture(&Path::new(FACES).join(DEFAULT_FACE), 128)),
            })
        })
        .collect();
    accounts.sort_by_key(|a| a.name.to_lowercase());
    accounts
}

/// A user name from a person's name: "Léa Martin" -> "lea-martin"
pub(super) fn user_from_name(name: &str, taken: &[String]) -> String {
    let mut base = String::new();
    for c in name.to_lowercase().chars() {
        let c = match c {
            'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => 'a',
            'ç' => 'c',
            'è' | 'é' | 'ê' | 'ë' => 'e',
            'ì' | 'í' | 'î' | 'ï' => 'i',
            'ñ' => 'n',
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' => 'o',
            'ù' | 'ú' | 'û' | 'ü' => 'u',
            'ý' | 'ÿ' => 'y',
            'ß' => 's',
            c if c.is_ascii_alphanumeric() => c,
            _ => '-',
        };
        if c != '-' || !base.ends_with('-') {
            base.push(c);
        }
    }
    let mut base = base.trim_matches('-').to_owned();
    base.truncate(24);
    if !base.starts_with(|c: char| c.is_ascii_lowercase()) {
        base = format!("child{base}");
    }
    let base = base.trim_end_matches('-').to_owned();
    let mut user = base.clone();
    let mut n = 2;
    while taken.contains(&user)
        || std::process::Command::new("getent")
            .args(["passwd", &user])
            .status()
            .is_ok_and(|status| status.success())
    {
        user = format!("{base}{n}");
        n += 1;
    }
    user
}

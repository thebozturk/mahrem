//! Basit argüman ayrıştırma: konumsal argümanlar, `--ad değer` ve bayraklar.

pub(crate) struct Args {
    pub(crate) positional: Vec<String>,
    options: Vec<(String, String)>,
    flags: Vec<String>,
}

impl Args {
    pub(crate) fn parse(raw: &[String], flags: &[&str]) -> Result<Self, String> {
        let mut a = Args {
            positional: Vec::new(),
            options: Vec::new(),
            flags: Vec::new(),
        };
        let mut it = raw.iter();
        while let Some(arg) = it.next() {
            if let Some(name) = arg.strip_prefix("--") {
                if flags.contains(&name) {
                    a.flags.push(name.to_owned());
                } else {
                    let value = it.next().ok_or(format!("--{name} değer ister"))?;
                    a.options.push((name.to_owned(), value.clone()));
                }
            } else {
                a.positional.push(arg.clone());
            }
        }
        Ok(a)
    }

    pub(crate) fn all(&self, name: &str) -> Vec<String> {
        self.options
            .iter()
            .filter(|(k, _)| k == name)
            .map(|(_, v)| v.clone())
            .collect()
    }

    pub(crate) fn one(&self, name: &str) -> Option<String> {
        self.all(name).pop()
    }

    pub(crate) fn need(&self, name: &str) -> Result<String, String> {
        self.one(name).ok_or(format!("--{name} gerekli"))
    }

    pub(crate) fn flag(&self, name: &str) -> bool {
        self.flags.iter().any(|f| f == name)
    }

    pub(crate) fn check_known(&self, known: &[&str]) -> Result<(), String> {
        match self
            .options
            .iter()
            .find(|(k, _)| !known.contains(&k.as_str()))
        {
            Some((k, _)) => Err(format!("bilinmeyen seçenek --{k}")),
            None => Ok(()),
        }
    }
}

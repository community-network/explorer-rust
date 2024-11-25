use anyhow::Ok;

const SYMBOLS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ123456789";

#[derive(Debug, Clone)]
pub struct ExperienceCode {
    experience_code: String,
}

impl From<String> for ExperienceCode {
    fn from(s: String) -> Self {
        Self {
            experience_code: s.to_uppercase(),
        }
    }
}

impl<'a> From<&'a str> for ExperienceCode {
    fn from(s: &'a str) -> Self {
        ExperienceCode {
            experience_code: s.to_string(),
        }
    }
}

impl From<ExperienceCode> for String {
    fn from(e: ExperienceCode) -> Self {
        e.experience_code
    }
}

impl<'a> From<&'a ExperienceCode> for &'a str {
    fn from(a: &'a ExperienceCode) -> Self {
        &a.experience_code
    }
}

impl ExperienceCode {
    pub fn from_usize(mut u: usize) -> anyhow::Result<ExperienceCode> {
        let base = SYMBOLS.len();

        let mut str_repr: String = "".to_string();

        while u > 0 {
            let res = (u / base, u % base);
            u = res.0;
            str_repr.push(SYMBOLS.chars().nth(res.1.try_into()?).unwrap());
        }

        let str_repr = str_repr.chars().rev().collect::<String>();
        Ok(Self {
            experience_code: format!("{:A>width$}", str_repr, width = str_repr.len() + 2),
        })
    }

    pub fn to_usize(&self) -> anyhow::Result<usize> {
        let mut u: usize = 0;
        let base = SYMBOLS.len();

        for (index, char) in self
            .experience_code
            .trim_start_matches("A")
            .chars()
            .enumerate()
        {
            u = u * base;
            u = u + SYMBOLS.find(char).ok_or(anyhow::anyhow!(
                "{} at {} not found in available symbols while parsing the experience code",
                char,
                index
            ))?;
        }
        Ok(u)
    }

    pub fn from_i32(i: i32) -> anyhow::Result<ExperienceCode> {
        if !i.is_positive() {
            return Err(anyhow::anyhow!("Integer representation must be positive."));
        }

        Ok(ExperienceCode::from_usize(i.try_into()?)?)
    }

    pub fn from_u32(i: u32) -> anyhow::Result<ExperienceCode> {
        Ok(ExperienceCode::from_usize(i.try_into()?)?)
    }

    pub fn is_valid(&self) -> anyhow::Result<bool> {
        if self.experience_code.len() < 3 {
            return Err(anyhow::anyhow!(
                "Experience code must be at least 3 characters long, not {}, {}",
                self.experience_code.len(),
                self.experience_code
            ));
        }

        if !self.experience_code.chars().all(|c| SYMBOLS.contains(c)) {
            return Err(anyhow::anyhow!(
                "Experience code contains invalid characters: {}",
                self.experience_code
            ));
        }
        Ok(true)
    }
}

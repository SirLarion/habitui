use std::fmt;

use serde::{Serialize, Deserialize, Serializer, Deserializer, de::{self, Visitor}};
use time::{OffsetDateTime, format_description::well_known::Iso8601};


#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Difficulty {
  TRIVIAL,
  EASY,
  MEDIUM,
  HARD
}

impl fmt::Display for Difficulty {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    use Difficulty::*;
    let diff = match self {
      TRIVIAL => "Trivial",
      EASY => "Easy",
      MEDIUM => "Medium",
      HARD => "Hard",
    };

    write!(f, "{}", diff)
  }
}

impl Into<f64> for Difficulty {
  fn into(self) -> f64 {
    match self {
      Difficulty::TRIVIAL => 0.1,
      Difficulty::EASY    => 1.0,
      Difficulty::MEDIUM  => 1.5,
      Difficulty::HARD    => 2.0
    }
  }
}

impl From<f64> for Difficulty {
  fn from(f: f64) -> Self {
    let eps = 0.01;
    if (0.1 - f).abs() < eps {
      Difficulty::TRIVIAL
    } else if (1.0 - f).abs() < eps {
      Difficulty::EASY
    } else if (1.5 - f).abs() < eps {
      Difficulty::MEDIUM
    } else {
      Difficulty::HARD
    }
  }
}

impl Difficulty {
  pub fn next(self) -> Self {
    use Difficulty::*;
    match self {
      TRIVIAL => EASY,
      EASY => MEDIUM,
      MEDIUM => HARD,
      HARD => TRIVIAL
    }
  }

  pub fn prev(self) -> Self {
    use Difficulty::*;
    match self {
      TRIVIAL => HARD,
      EASY => TRIVIAL,
      MEDIUM => EASY,
      HARD => MEDIUM
    }
  }
}

impl Serialize for Difficulty {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer
  {
    serializer.serialize_f64(Into::into(*self))
  }
}

struct F32Visitor;

impl<'de> Visitor<'de> for F32Visitor {
  type Value = f64;

  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    formatter.write_str("a floating point number within the following: 0.1, 1.0, 1.5, 2.0")
  }

  fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
      E: de::Error, 
  {
    if value <= 2 {
      Ok(value as f64)
    } else {
      Err(E::custom(format!("priority is outside of range {}", value)))
    }
  }

  fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
  where
      E: de::Error,
  {
      Ok(value)
  }
}

impl<'de> Deserialize<'de> for Difficulty {
  fn deserialize<D>(deserializer: D) -> Result<Difficulty, D::Error>
  where
    D: Deserializer<'de>
  {
    let float = deserializer.deserialize_f32(F32Visitor)?;
    Ok(Difficulty::from(float))
  }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Eq, Hash)]
pub struct TaskId(String);

impl fmt::Display for TaskId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl TaskId {
  pub fn empty() -> Self {
    Self("".into())
  }

  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub struct SubTask {
  pub text: String,
  pub completed: bool
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Task {
  #[serde(rename = "_id")]
  pub id: TaskId,
  pub text: String,
  #[serde(rename = "type")]
  pub task_type: String,
  #[serde(rename = "priority")]
  pub difficulty: Difficulty,
  pub notes: Option<String>,
  #[serde(default,
    deserialize_with = "time::serde::iso8601::option::deserialize", 
    serialize_with = "time::serde::iso8601::option::serialize"
  )]
  pub date: Option<OffsetDateTime>,
  pub checklist: Option<Vec<SubTask>>,
}

impl Default for Task {
  fn default() -> Self {
    Self {
      id: TaskId::empty(),
      text: "".into(),
      notes: None,
      task_type: "todo".into(),
      difficulty: Difficulty::EASY, 
      date: None,
      checklist: None,
    }
  }
}

impl fmt::Display for Task {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{:48}{:7}", &self.text, &self.difficulty)?;
    let _ = &self.notes.clone().map(|n| write!(f, "\n{}", n));
    let _ = &self.date.clone().map(|d| write!(f, "\n{}", d.format(&Iso8601::DATE).unwrap()));

    if let Some(subtasks) = &self.checklist {
      for SubTask { text, completed } in subtasks {
        let check = if *completed { "✅" } else { 
          if cfg!(feature = "dark-mode") {
            "⬛"
          } else {
            "⬜" 
          }
        };
        write!(f, "\n{check} {text}")?;
      }
    }
    write!(f, "\n")
  }
}

use crate::error::ScadaFoundryError;

pub enum IccpDataPointIdentifier {
    Vcc { name: String },
    Icc { domain: String, name: String },
}

impl TryFrom<&ApiIccpDataPointIdentifier> for IccpDataPointIdentifier {
    type Error = ScadaFoundryError;

    fn try_from(value: &ApiIccpDataPointIdentifier) -> Result<Self, Self::Error> {
        value.clone().try_into()
    }
}

impl TryFrom<ApiIccpDataPointIdentifier> for IccpDataPointIdentifier {
    type Error = ScadaFoundryError;

    fn try_from(value: ApiIccpDataPointIdentifier) -> Result<Self, Self::Error> {
        match (value.kind.as_ref(), value.domain, value.name) {
            ("Vcc", None, name) => Ok(Self::Vcc { name }),
            ("Vcc", _, _) => Err(ScadaFoundryError::ApplicationError("VCC scoped MMS Object Name cannot have a domain".into())),
            ("Icc", Some(domain), name) => Ok(Self::Icc { domain, name }),
            ("Icc", _, _) => Err(ScadaFoundryError::ApplicationError("ICC scoped MMS Object Name cannot have a domain".into())),
            (x, _, _) => Err(ScadaFoundryError::ApplicationError(format!("only a kind of Vcc or Icc is supported as an MMS object Name Scope but got {x}"))),
        }
    }
}

pub struct ApiIccpDataPointIdentifier {
    kind: String,               // Vcc / Icc
    pub domain: Option<String>, // Mandatory for Icc only. Blank or optional for Vcc
    pub name: String,
}

impl ApiIccpDataPointIdentifier {
    pub fn new_vcc(name: String) -> Self {
        Self { kind: "Vcc".into(), domain: None, name }
    }

    pub fn new_icc(domain: String, name: String) -> Self {
        Self { kind: "Icc".into(), domain: Some(domain), name }
    }
}

impl From<&IccpDataPointIdentifier> for ApiIccpDataPointIdentifier {
    fn from(value: &IccpDataPointIdentifier) -> Self {
        value.clone().into()
    }
}

impl From<IccpDataPointIdentifier> for ApiIccpDataPointIdentifier {
    fn from(value: IccpDataPointIdentifier) -> Self {
        match value {
            IccpDataPointIdentifier::Vcc { name } => Self::new_vcc(name),
            IccpDataPointIdentifier::Icc { domain, name } => Self::new_icc(domain, name),
        }
    }
}

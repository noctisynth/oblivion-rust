use crate::exceptions::OblivionException;

pub enum Router {
    Route(String),
    Startswith(String),
}

pub struct Route {
    olps: String,
}

impl Route {
    pub fn new(olps: &str) -> Result<Self, OblivionException> {
        Ok(Self {
            olps: olps.to_string(),
        })
    }

    pub fn verify(&mut self, olps: &str) -> bool {
        olps.trim_end_matches("/") == self.olps.trim_end_matches("/")
    }

    pub fn get_olps(&mut self) -> String {
        self.olps.clone()
    }
}

pub struct Startswith {
    olps: String,
}

impl Startswith {
    pub fn new(olps: &str) -> Result<Self, OblivionException> {
        Ok(Self {
            olps: olps.to_string(),
        })
    }

    pub fn verify(&mut self, olps: &str) -> bool {
        olps.trim_end_matches("/") == self.olps.trim_end_matches("/")
    }

    pub fn get_olps(&mut self) -> String {
        self.olps.clone()
    }
}

impl Router {
    pub fn verify(&mut self, olps: &str) -> Result<bool, OblivionException> {
        match self {
            Self::Route(info) => {
                let mut route = Route::new(&info)?;
                Ok(route.verify(olps))
            }
            Self::Startswith(info) => {
                let mut route = Startswith::new(&info)?;
                Ok(route.verify(olps))
            }
        }
    }

    pub fn get_olps(&mut self) -> String {
        match self {
            Self::Route(info) => info.to_string(),
            Self::Startswith(info) => info.to_string(),
        }
    }
}

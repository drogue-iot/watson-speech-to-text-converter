use cloudevents::{AttributesReader, Event};
use envconfig::Envconfig;
use std::collections::HashSet;
use std::iter::FromIterator;

#[derive(Clone, Debug, Default, Envconfig)]
pub struct FilterConfig {
    #[envconfig(from = "ONLY_TYPES", default = "")]
    pub types: String,
    #[envconfig(from = "ONLY_SUBJECTS", default = "")]
    pub subjects: String,
    #[envconfig(from = "ONLY_DATACONTENTTYPES", default = "")]
    pub data_content_types: String,
}

#[derive(Clone, Debug)]
pub struct Filter {
    types: Option<HashSet<String>>,
    subjects: Option<HashSet<String>>,
    data_content_types: Option<HashSet<String>>,
}

impl From<FilterConfig> for Filter {
    fn from(config: FilterConfig) -> Self {
        Filter {
            types: Self::split(&config.types),
            subjects: Self::split(&config.subjects),
            data_content_types: Self::split(&config.data_content_types),
        }
    }
}

impl Filter {
    fn split(str: &String) -> Option<HashSet<String>> {
        if str.is_empty() {
            None
        } else {
            Some(HashSet::from_iter(
                str.split([',', ' '].as_ref()).map(|s| s.to_string()),
            ))
        }
    }

    pub fn test(&self, event: &Event) -> bool {
        if !self.pass(&self.types, Some(event.ty())) {
            return false;
        }
        if !self.pass(&self.subjects, event.subject()) {
            return false;
        }
        if !self.pass(&self.data_content_types, event.datacontenttype()) {
            return false;
        }

        true
    }

    fn pass(&self, filter: &Option<HashSet<String>>, value: Option<&str>) -> bool {
        match (value, filter) {
            // no value, no filter -> pass
            (_, None) => true,
            // no value, filter present -> reject
            (None, Some(_)) => false,
            // value and filter -> check
            (Some(value), Some(filter)) => filter.contains(value),
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use cloudevents::EventBuilder;

    #[test]
    fn test_filter_1() {
        let filter: Filter = FilterConfig {
            types: "a, b, c".into(),
            ..Default::default()
        }
        .into();

        let event_pass = cloudevents::EventBuilderV10::default()
            .ty("b")
            .build()
            .unwrap();
        let event_reject = cloudevents::EventBuilderV10::default()
            .ty("x")
            .build()
            .unwrap();

        assert!(filter.test(&event_pass));
        assert!(!filter.test(&event_reject));
    }
}

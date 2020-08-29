use std::borrow::Borrow;
use std::borrow::Cow;
use std::{
    iter::FromIterator,
    path::{Path, PathBuf},
};

use futures::stream::{Stream, StreamExt};

use fluent_bundle::FluentResource;
use fluent_bundle::{FluentArgs, FluentBundle};

pub type GenerateBundlesSync<R> =
    dyn for<'iter> FnMut(
        &'iter mut dyn Iterator<Item = &'iter Path>,
    ) -> Box<dyn Iterator<Item = FluentBundle<R>> + 'iter>;

pub struct Localization<R> {
    resource_ids: Vec<PathBuf>,
    generate_bundles_sync: Option<Box<GenerateBundlesSync<R>>>,
}

impl<R> Localization<R> {
    pub fn new<P>(resource_ids: impl IntoIterator<Item = P>) -> Self
    where
        P: AsRef<Path>,
    {
        let resource_ids: Vec<PathBuf> = resource_ids
            .into_iter()
            .map(move |p| p.as_ref().to_path_buf())
            .collect();
        Self {
            resource_ids,
            generate_bundles_sync: None,
        }
    }

    pub fn generate_bundles_sync<F>(&mut self, generate_bundles_sync: F)
    where
        for<'a> F: FnMut(
                &'a mut dyn Iterator<Item = &Path>,
            ) -> Box<dyn Iterator<Item = FluentBundle<R>> + 'a>
            + 'static,
    {
        self.generate_bundles_sync = Some(Box::new(generate_bundles_sync));
    }

    // pub struct Localization<R, I, S>
    // where
    //     I: Iterator<Item = FluentBundle<R>>,
    //     S: Stream<Item = FluentBundle<R>>
    // {
    //     resource_ids: Vec<PathBuf>,
    //     generate_bundles: Option<fn(Vec<PathBuf>) -> S>,
    //     generate_bundles_sync: Option<fn(Vec<PathBuf>) -> I>,
    // }
    //
    // impl<'l, R: 'l, I, S> Localization<R, I, S>
    // where
    //     I: Iterator<Item = FluentBundle<R>>,
    //     S: Stream<Item = FluentBundle<R>>,
    // {
    //     pub fn new<F, A>(
    //         resource_ids: Vec<PathBuf>,
    //         generate_bundles: Option<fn(Vec<PathBuf>) -> S>,
    //         generate_bundles_sync: Option<fn(Vec<PathBuf>) -> I>,
    //     ) -> Self {
    //         Self {
    //             resource_ids,
    //             generate_bundles,
    //             generate_bundles_sync,
    //         }
    //     }

    // pub async fn format_value(&mut self, id: &'l str, args: Option<&'l FluentArgs<'_>>) -> Cow<'l, str>
    // where
    //     R: Borrow<FluentResource>,
    // {
    //     let bundles = self.generate_bundles.unwrap()(self.resource_ids.clone());
    //     let mut i = Box::pin(bundles);
    //     while let Some(bundle) = (i.next()).await {
    //         if let Some(msg) = bundle.get_message(id) {
    //             if let Some(pattern) = msg.value {
    //                 let mut errors = vec![];
    //                 let val: Cow<'_, str> = bundle.format_pattern(pattern, args, &mut errors);
    //                 return val.to_string().into();
    //             }
    //         }
    //     }
    //     "Missing".into()
    // }
    //
    // pub fn format_value_sync(&mut self, id: &'l str, args: Option<&'l FluentArgs>) -> Cow<'l, str>
    // where
    //     R: Borrow<FluentResource>,
    // {
    //     let bundles = self.generate_bundles_sync.unwrap()(self.resource_ids.clone());
    //     for bundle in bundles {
    //         if let Some(msg) = bundle.get_message(id) {
    //             if let Some(pattern) = msg.value {
    //                 let mut errors = vec![];
    //                 let val: Cow<'_, str> = bundle.format_pattern(pattern, args, &mut errors);
    //                 return val.to_string().into();
    //             }
    //         }
    //     }
    //     "Missing".into()
    // }

    pub fn format_value_sync(&mut self, id: &str, args: Option<&FluentArgs>) -> Cow<str>
    where
        R: Borrow<FluentResource>,
    {
        let mut iter = self.resource_ids.iter().map(|p| p.as_path());
        if let Some(ref mut generate_bundles_sync) = self.generate_bundles_sync {
            let bundles = generate_bundles_sync(&mut iter);
            for bundle in bundles {
                if let Some(msg) = bundle.get_message(id) {
                    if let Some(pattern) = msg.value {
                        let mut errors = vec![];
                        let val: Cow<'_, str> = bundle.format_pattern(pattern, args, &mut errors);
                        return val.to_string().into();
                    }
                }
            }
        }
        "Missing".into()
    }
}

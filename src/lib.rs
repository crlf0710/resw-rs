#![cfg_attr(feature = "unstable", feature(specialization))]
#![allow(dead_code)]
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::io;
use std::fmt;
use std::path::Path;
use winapi::shared::minwindef::DWORD;
use winapi::shared::minwindef::WORD;

type CowStr = Cow<'static, str>;
type CowPath = Cow<'static, Path>;

#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Debug)]
pub struct Lang(WORD, WORD);

pub mod lang {
    use super::Lang;
    use winapi::shared::ntdef::*;

    pub const LANG_ENU: Lang = Lang(LANG_ENGLISH, SUBLANG_ENGLISH_US);
    pub const PRESET_LANG_1: &[Lang] = &[LANG_ENU];

    pub const LANG_CHS: Lang = Lang(LANG_CHINESE, SUBLANG_CHINESE_SIMPLIFIED);
    pub const LANG_CHT: Lang = Lang(LANG_CHINESE, SUBLANG_CHINESE_TRADITIONAL);
    pub const LANG_DEU: Lang = Lang(LANG_GERMAN, SUBLANG_GERMAN);
    pub const LANG_ESN: Lang = Lang(LANG_SPANISH, SUBLANG_SPANISH);
    pub const LANG_FRA: Lang = Lang(LANG_FRENCH, SUBLANG_FRENCH);
    pub const LANG_ITA: Lang = Lang(LANG_ITALIAN, SUBLANG_ITALIAN);
    pub const LANG_JPN: Lang = Lang(LANG_JAPANESE, SUBLANG_JAPANESE_JAPAN);
    pub const LANG_KOR: Lang = Lang(LANG_KOREAN, SUBLANG_KOREAN);

    pub const PRESET_LANG_9: &[Lang] = &[
        LANG_ENU, LANG_CHS, LANG_CHT, LANG_DEU, LANG_ESN, LANG_FRA, LANG_ITA, LANG_JPN, LANG_KOR,
    ];

    pub const LANG_RUS: Lang = Lang(LANG_RUSSIAN, SUBLANG_RUSSIAN_RUSSIA);

    pub const PRESET_LANG_10: &[Lang] = &[
        LANG_ENU, LANG_CHS, LANG_CHT, LANG_DEU, LANG_ESN, LANG_FRA, LANG_ITA, LANG_JPN, LANG_KOR,
        LANG_RUS,
    ];

    pub const LANG_CSY: Lang = Lang(LANG_CZECH, SUBLANG_CZECH_CZECH_REPUBLIC);
    pub const LANG_PLK: Lang = Lang(LANG_POLISH, SUBLANG_POLISH_POLAND);
    pub const LANG_PTB: Lang = Lang(LANG_PORTUGUESE, SUBLANG_PORTUGUESE_BRAZILIAN);
    pub const LANG_TRK: Lang = Lang(LANG_TURKISH, SUBLANG_TURKISH_TURKEY);

    pub const PRESET_LANG_14: &[Lang] = &[
        LANG_ENU, LANG_CHS, LANG_CHT, LANG_CSY, LANG_DEU, LANG_ESN, LANG_FRA, LANG_ITA, LANG_JPN,
        LANG_KOR, LANG_PLK, LANG_PTB, LANG_RUS, LANG_TRK,
    ];
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug)]
pub struct Id(WORD);

impl From<WORD> for Id {
    fn from(v: WORD) -> Self {
        Id(v)
    }
}

impl From<isize> for Id {
    fn from(v: isize) -> Self {
        let v: WORD = match v {
            -1..=0xFFFF => v as u16,
            _ => panic!("id out of bound, expected u16, actual value = {}", v),
        };
        Id(v)
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0).map_err(|_| fmt::Error)?;
        Ok(())
    }
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug)]
pub enum IdOrName {
    Id(Id),
    Name(CowStr),
}

impl From<WORD> for IdOrName {
    fn from(v: WORD) -> Self {
        IdOrName::Id(Id(v))
    }
}

impl From<isize> for IdOrName {
    fn from(v: isize) -> Self {
        IdOrName::Id(Id::from(v))
    }
}

impl From<String> for IdOrName {
    fn from(v: String) -> Self {
        IdOrName::Name(Cow::Owned(v))
    }
}

#[cfg(not(feature = "unstable"))]
impl<'a> From<&'a str> for IdOrName {
    fn from(v: &'a str) -> Self {
        IdOrName::Name(Cow::Owned(v.to_owned()))
    }
}

#[cfg(feature = "unstable")]
default impl<'a> From<&'a str> for IdOrName {
    fn from(v: &'a str) -> Self {
        IdOrName::Name(Cow::Owned(v.to_owned()))
    }
}

#[cfg(feature = "unstable")]
default impl<'a> From<&'a str> for IdOrName
where
    'a: 'static,
{
    fn from(v: &'a str) -> Self {
        IdOrName::Name(Cow::Borrowed(v))
    }
}

pub const NOT_USEFUL_ID: Id = Id(-1 as _);

pub struct Build {
    resources: BTreeMap<Lang, Vec<(IdOrName, Box<dyn Resource>)>>,
}

impl Build {
    pub fn new(languages: &[Lang]) -> Self {
        let mut build = Build {
            resources: BTreeMap::new(),
        };
        for language in languages.iter().cloned() {
            build.resources.insert(language, Vec::new());
        }
        build
    }

    pub fn with_one_language() -> Self {
        Self::new(lang::PRESET_LANG_1)
    }

    pub fn with_one_or_two_languages(l: Lang) -> Self {
        if l == lang::LANG_ENU {
            Self::with_one_language()
        } else {
            Self::with_two_languages(l)
        }
    }

    pub fn with_two_languages(l: Lang) -> Self {
        assert_ne!(l, lang::LANG_ENU);
        Self::new(&[lang::LANG_ENU, l])
    }

    pub fn with_nine_languages() -> Self {
        Self::new(lang::PRESET_LANG_9)
    }

    pub fn resource(
        mut self,
        id_or_name: impl Into<IdOrName>,
        resource: impl Resource + Clone,
    ) -> Self {
        let id_or_name: IdOrName = id_or_name.into();
        for (_lang, lang_specific_resources) in self.resources.iter_mut() {
            lang_specific_resources.push((id_or_name.clone(), Box::new(resource.clone())));
        }
        self
    }

    pub fn lang_specific_resource(
        mut self,
        language: Lang,
        id_or_name: impl Into<IdOrName>,
        resource: impl Resource,
    ) -> Self {
        let id_or_name: IdOrName = id_or_name.into();
        let lang_specific_resources = self.resources.entry(language).or_default();
        lang_specific_resources.push((id_or_name, Box::new(resource)));
        self
    }
}

pub trait Resource: 'static {
    fn write_script_segment(
        &self,
        _w: &mut dyn io::Write,
        _l: Lang,
        _id_or_name: IdOrName,
    ) -> io::Result<()> {
        unimplemented!()
    }
}

#[macro_use]
pub mod resource {
    use crate::{CowPath, Resource};
    use std::path::Path;
    use std::rc::Rc;

    fn create_path_only_resource_from_file<T, R: From<CowPath>>(
        path: impl AsRef<Path>,
        ty: impl FnOnce(Rc<R>) -> T,
    ) -> T {
        use std::borrow::Cow;
        ty(Rc::new(R::from(Cow::Owned(path.as_ref().to_owned()))))
    }

    macro_rules! define_path_only_resource {
        ($type_name:ident, $res_type_keyword:literal) => {
            #[derive(Clone)]
            pub struct $type_name(Rc<CowPath>);

            impl $type_name {
                pub fn from_file(path: impl AsRef<Path>) -> Self {
                    create_path_only_resource_from_file(path, $type_name)
                }
            }

            impl Resource for $type_name {
                fn write_script_segment(
                    &self,
                    w: &mut dyn std::io::Write,
                    l: crate::Lang,
                    id_or_name: crate::IdOrName,
                ) -> Result<(), std::io::Error> {
                    crate::codegen::write_path_only_resource(
                        w,
                        l,
                        id_or_name,
                        $res_type_keyword,
                        self.0.as_ref(),
                    )?;
                    Ok(())
                }
            }
        };
    }

    macro_rules! define_builder_generated_resource {
        ($type_name:ident, $data_type:path, $builder_type:path, $res_type_keyword:literal) => {
            #[derive(Clone)]
            pub struct $type_name(pub(crate) Rc<$data_type>);

            impl $type_name {
                pub(crate) const TYPE_KEYWORD: &'static str = $res_type_keyword;

                pub fn from_builder() -> $builder_type {
                    <$builder_type as crate::PrivDefault>::priv_default()
                }
            }

            impl Resource for $type_name {
                fn write_script_segment(
                    &self,
                    w: &mut dyn std::io::Write,
                    l: crate::Lang,
                    id_or_name: crate::IdOrName,
                ) -> Result<(), std::io::Error> {
                    if self.0.as_ref().is_missing_for_lang(l) {
                        return Ok(())
                    }
                    crate::codegen::write_resource_header(w, l, id_or_name, $res_type_keyword)?;
                    self.0.as_ref().write_resource_header_extras(w, l)?;
                    write!(w, "\n")?;
                    self.0.as_ref().write_resource_segment(w, l)?;
                    Ok(())
                }
            }
        };
    }

    macro_rules! builder_implement_priv_default {
        ($builder_type:path) => {
            impl crate::PrivDefault for $builder_type {
                fn priv_default() -> Self {
                    $builder_type(Default::default())
                }
            }
        };
    }

    macro_rules! builder_extra_info_methods {
        ($builder_type:path) => {
            impl $builder_type {
                pub fn extra_info(
                    mut self,
                    characteristics: Option<DWORD>,
                    version: Option<DWORD>,
                ) -> Self {
                    use crate::ExtraInfo;
                    let fallback_items = (self.0).0.access_fallback_mut();
                    fallback_items.extra_info = Some(ExtraInfo {
                        characteristics,
                        version,
                    });
                    self
                }

                pub fn lang_specific_extra_info(
                    mut self,
                    lang: crate::Lang,
                    characteristics: Option<DWORD>,
                    version: Option<DWORD>,
                ) -> Self {
                    use crate::ExtraInfo;
                    let lang_items = (self.0).0.access_lang_specific_mut(lang);
                    lang_items.extra_info = Some(ExtraInfo {
                        characteristics,
                        version,
                    });
                    self
                }
            }
        };
    }

    macro_rules! builder_extra_info_methods2 {
        ($builder_type:path) => {
            impl $builder_type {
                pub fn extra_info(
                    mut self,
                    characteristics: Option<DWORD>,
                    version: Option<DWORD>,
                ) -> Self {
                    use crate::ExtraInfo;
                    (self.0).extra_info.insert_fallback(ExtraInfo {
                        characteristics,
                        version,
                    });
                    self
                }

                pub fn lang_specific_extra_info(
                    mut self,
                    lang: crate::Lang,
                    characteristics: Option<DWORD>,
                    version: Option<DWORD>,
                ) -> Self {
                    use crate::ExtraInfo;
                    (self.0).extra_info.insert_lang_specific(
                        lang,
                        ExtraInfo {
                            characteristics,
                            version,
                        },
                    );
                    self
                }
            }
        };
    }

    macro_rules! builder_build_method {
        ($builder_type:path, $type_name:path) => {
            impl $builder_type {
                pub fn build(self) -> $type_name {
                    use std::rc::Rc;
                    $type_name(Rc::new(self.0))
                }
            }
        };
    }

    macro_rules! bitflags_bitor_method {
        ($bitflags_type:path) => {
            impl std::ops::BitOr for $bitflags_type {
                type Output = $bitflags_type;

                fn bitor(self, rhs: $bitflags_type) -> $bitflags_type {
                    $bitflags_type(self.0 | rhs.0)
                }
            }
        };
    }

    macro_rules! define_builder_or_path_generated_resource {
        ($type_name:ident, $data_type:path, $builder_type:path) => {
            #[derive(Clone)]
            pub struct $type_name(pub(crate) Rc<$data_type>);

            impl $type_name {
                pub fn from_builder() -> $builder_type {
                    <$builder_type as crate::PrivDefault>::priv_default()
                }

                pub fn from_file(path: impl AsRef<Path>) -> Self {
                    create_path_only_resource_from_file(path, $type_name)
                }
            }

            impl Resource for $type_name {}
        };
    }

    macro_rules! unimplemented_resouce_data_write_segment {
        ($type_name:ident) => {
            impl $type_name {
                pub(crate) fn is_missing_for_lang(&self, l: crate::Lang) -> bool {
                    true
                }

                pub(crate) fn write_resource_header_extras(&self, w: &mut dyn std::io::Write, l: crate::Lang) -> Result<(), std::io::Error> {
                    unimplemented!()
                }

                pub(crate) fn write_resource_segment(&self, w: &mut dyn std::io::Write, l: crate::Lang) -> Result<(), std::io::Error> {
                    unimplemented!()
                }
            }
        };
    }

    define_path_only_resource!(Bitmap, "BITMAP");
    define_path_only_resource!(Cursor, "CURSOR");
    define_path_only_resource!(Font, "FONT");
    define_path_only_resource!(HTML, "HTML");
    define_path_only_resource!(Icon, "ICON");
    define_path_only_resource!(MessageTable, "MESSAGETABLE");

    define_builder_generated_resource!(
        StringTable,
        crate::string_table::StringTableData,
        crate::string_table::StringTableBuilder,
        "STRINGTABLE"
    );

    define_builder_generated_resource!(
        Accelerators,
        crate::accelerators::AcceleratorsData,
        crate::accelerators::AcceleratorsBuilder,
        "ACCELERATORS"
    );

    define_builder_generated_resource!(
        Menu,
        crate::menu::MenuData,
        crate::menu::MenuBuilder,
        "MENUEX"
    );

    define_builder_generated_resource!(
        Dialog,
        crate::dialog::DialogData,
        crate::dialog::DialogBuilder,
        "DIALOGEX"
    );

    define_builder_generated_resource!(
        VersionInfo,
        crate::version_info::VersionInfoData,
        crate::version_info::VersionInfoBuilder,
        "VERSIONINFO"
    );

    define_builder_generated_resource!(
        RcInline,
        crate::rc_inline::RcInlineData,
        crate::rc_inline::RcInlineBuilder,
        "RCDATA"
    );

    define_builder_or_path_generated_resource!(
        UserDefined,
        crate::user_defined::UserDefinedData,
        crate::user_defined::UserDefinedBuilder
    );

    // we won't support:
    // obsolete items: plugplay vxd
    // special items: textinclude typelib
}

struct OptionLangSpecific<T>(BTreeMap<Option<Lang>, T>);

impl<T> OptionLangSpecific<T> {
    fn access_lang_specific_mut(&mut self, lang: Lang) -> &mut T
    where
        T: Default,
    {
        self.0.entry(Some(lang)).or_default()
    }

    fn access_fallback_mut(&mut self) -> &mut T
    where
        T: Default,
    {
        self.0.entry(None).or_default()
    }

    fn insert_lang_specific(&mut self, lang: Lang, v: T) {
        self.0.insert(Some(lang), v);
    }

    fn insert_fallback(&mut self, v: T) {
        self.0.insert(None, v);
    }

    fn get(&self, lang: Lang) -> Option<&T> {
        if let Some(v) = self.0.get(&Some(lang)) {
            Some(v)
        } else if let Some(v) = self.0.get(&None) {
            Some(v)
        } else {
            None
        }
    }
}

impl<T> Default for OptionLangSpecific<T> {
    fn default() -> Self {
        OptionLangSpecific(BTreeMap::default())
    }
}

pub struct ExtraInfo {
    pub characteristics: Option<DWORD>,
    pub version: Option<DWORD>,
}

pub struct MultiLangText(OptionLangSpecific<CowStr>);

impl MultiLangText {
    fn empty() -> Self {
        MultiLangText(OptionLangSpecific::default())
    }

    pub fn lang(mut self, lang: Lang, str: impl Into<CowStr>) -> Self {
        self.0.insert_lang_specific(lang, str.into());
        self
    }
}

impl<T> From<T> for MultiLangText where T: Into<CowStr> {
    fn from(v: T) -> Self {
        let mut r = Self::empty();
        r.0.insert_fallback(v.into());
        r
    }
}

trait PrivDefault {
    fn priv_default() -> Self;
}

pub mod string_table {
    use crate::{ExtraInfo, Id, Lang, OptionLangSpecific};
    use winapi::shared::minwindef::DWORD;

    #[derive(Default)]
    struct StringTableItems {
        extra_info: Option<ExtraInfo>,
        strings: Vec<(Id, String)>,
    }

    #[derive(Default)]
    pub(crate) struct StringTableData(OptionLangSpecific<StringTableItems>);

    pub struct StringTableBuilder(StringTableData);

    builder_implement_priv_default!(StringTableBuilder);
    builder_extra_info_methods!(StringTableBuilder);
    builder_build_method!(StringTableBuilder, crate::resource::StringTable);

    impl StringTableBuilder {
        pub fn string(mut self, id: impl Into<Id>, string: impl AsRef<str>) -> Self {
            let id = id.into();
            let string = string.as_ref().to_owned();
            let fallback_items = (self.0).0.access_fallback_mut();
            fallback_items.strings.push((id, string));
            self
        }

        pub fn lang_specific_string(
            mut self,
            lang: Lang,
            id: impl Into<Id>,
            string: impl AsRef<str>,
        ) -> Self {
            let id = id.into();
            let string = string.as_ref().to_owned();
            let lang_items = (self.0).0.access_lang_specific_mut(lang);
            lang_items.strings.push((id, string));
            self
        }
    }

    unimplemented_resouce_data_write_segment!(StringTableData);
}

pub mod accelerators {
    use crate::{ExtraInfo, Id, Lang, OptionLangSpecific};
    use std::fmt;
    use winapi::ctypes::c_int;
    use winapi::shared::minwindef::DWORD;
    use winapi::um::winuser;

    #[derive(Clone, Copy)]
    pub struct ASCIIKey(u8);

    impl ASCIIKey {
        pub fn ascii_key(v: u8) -> ASCIIKey {
            match v {
                32u8..=126u8 => Some(ASCIIKey(v)),
                _ => None,
            }
            .expect("provided u8 value is not ascii key")
        }
    }

    impl fmt::Display for ASCIIKey {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.0).map_err(|_| fmt::Error)?;
            Ok(())
        }
    }

    #[derive(Clone, Copy)]
    pub struct VirtKey(c_int);

    impl VirtKey {
        pub const NUM_0: VirtKey = VirtKey(0x30);
        pub const NUM_1: VirtKey = VirtKey(0x31);
        pub const NUM_2: VirtKey = VirtKey(0x32);
        pub const NUM_3: VirtKey = VirtKey(0x33);
        pub const NUM_4: VirtKey = VirtKey(0x34);
        pub const NUM_5: VirtKey = VirtKey(0x35);
        pub const NUM_6: VirtKey = VirtKey(0x36);
        pub const NUM_7: VirtKey = VirtKey(0x37);
        pub const NUM_8: VirtKey = VirtKey(0x38);
        pub const NUM_9: VirtKey = VirtKey(0x39);
        pub const LETTER_A: VirtKey = VirtKey(0x41);
        pub const LETTER_B: VirtKey = VirtKey(0x42);
        pub const LETTER_C: VirtKey = VirtKey(0x43);
        pub const LETTER_D: VirtKey = VirtKey(0x44);
        pub const LETTER_E: VirtKey = VirtKey(0x45);
        pub const LETTER_F: VirtKey = VirtKey(0x46);
        pub const LETTER_G: VirtKey = VirtKey(0x47);
        pub const LETTER_H: VirtKey = VirtKey(0x48);
        pub const LETTER_I: VirtKey = VirtKey(0x49);
        pub const LETTER_J: VirtKey = VirtKey(0x4A);
        pub const LETTER_K: VirtKey = VirtKey(0x4B);
        pub const LETTER_L: VirtKey = VirtKey(0x4C);
        pub const LETTER_M: VirtKey = VirtKey(0x4D);
        pub const LETTER_N: VirtKey = VirtKey(0x4E);
        pub const LETTER_O: VirtKey = VirtKey(0x4F);
        pub const LETTER_P: VirtKey = VirtKey(0x50);
        pub const LETTER_Q: VirtKey = VirtKey(0x51);
        pub const LETTER_R: VirtKey = VirtKey(0x52);
        pub const LETTER_S: VirtKey = VirtKey(0x53);
        pub const LETTER_T: VirtKey = VirtKey(0x54);
        pub const LETTER_U: VirtKey = VirtKey(0x55);
        pub const LETTER_V: VirtKey = VirtKey(0x56);
        pub const LETTER_W: VirtKey = VirtKey(0x57);
        pub const LETTER_X: VirtKey = VirtKey(0x58);
        pub const LETTER_Y: VirtKey = VirtKey(0x59);
        pub const LETTER_Z: VirtKey = VirtKey(0x5A);
        pub const LBUTTON: VirtKey = VirtKey(winuser::VK_LBUTTON);
        pub const RBUTTON: VirtKey = VirtKey(winuser::VK_RBUTTON);
        pub const CANCEL: VirtKey = VirtKey(winuser::VK_CANCEL);
        pub const MBUTTON: VirtKey = VirtKey(winuser::VK_MBUTTON);
        pub const XBUTTON1: VirtKey = VirtKey(winuser::VK_XBUTTON1);
        pub const XBUTTON2: VirtKey = VirtKey(winuser::VK_XBUTTON2);
        pub const BACK: VirtKey = VirtKey(winuser::VK_BACK);
        pub const TAB: VirtKey = VirtKey(winuser::VK_TAB);
        pub const CLEAR: VirtKey = VirtKey(winuser::VK_CLEAR);
        pub const RETURN: VirtKey = VirtKey(winuser::VK_RETURN);
        pub const SHIFT: VirtKey = VirtKey(winuser::VK_SHIFT);
        pub const CONTROL: VirtKey = VirtKey(winuser::VK_CONTROL);
        pub const MENU: VirtKey = VirtKey(winuser::VK_MENU);
        pub const PAUSE: VirtKey = VirtKey(winuser::VK_PAUSE);
        pub const CAPITAL: VirtKey = VirtKey(winuser::VK_CAPITAL);
        pub const KANA: VirtKey = VirtKey(winuser::VK_KANA);
        pub const HANGEUL: VirtKey = VirtKey(winuser::VK_HANGEUL);
        pub const HANGUL: VirtKey = VirtKey(winuser::VK_HANGUL);
        pub const JUNJA: VirtKey = VirtKey(winuser::VK_JUNJA);
        pub const FINAL: VirtKey = VirtKey(winuser::VK_FINAL);
        pub const HANJA: VirtKey = VirtKey(winuser::VK_HANJA);
        pub const KANJI: VirtKey = VirtKey(winuser::VK_KANJI);
        pub const ESCAPE: VirtKey = VirtKey(winuser::VK_ESCAPE);
        pub const CONVERT: VirtKey = VirtKey(winuser::VK_CONVERT);
        pub const NONCONVERT: VirtKey = VirtKey(winuser::VK_NONCONVERT);
        pub const ACCEPT: VirtKey = VirtKey(winuser::VK_ACCEPT);
        pub const MODECHANGE: VirtKey = VirtKey(winuser::VK_MODECHANGE);
        pub const SPACE: VirtKey = VirtKey(winuser::VK_SPACE);
        pub const PRIOR: VirtKey = VirtKey(winuser::VK_PRIOR);
        pub const NEXT: VirtKey = VirtKey(winuser::VK_NEXT);
        pub const END: VirtKey = VirtKey(winuser::VK_END);
        pub const HOME: VirtKey = VirtKey(winuser::VK_HOME);
        pub const LEFT: VirtKey = VirtKey(winuser::VK_LEFT);
        pub const UP: VirtKey = VirtKey(winuser::VK_UP);
        pub const RIGHT: VirtKey = VirtKey(winuser::VK_RIGHT);
        pub const DOWN: VirtKey = VirtKey(winuser::VK_DOWN);
        pub const SELECT: VirtKey = VirtKey(winuser::VK_SELECT);
        pub const PRINT: VirtKey = VirtKey(winuser::VK_PRINT);
        pub const EXECUTE: VirtKey = VirtKey(winuser::VK_EXECUTE);
        pub const SNAPSHOT: VirtKey = VirtKey(winuser::VK_SNAPSHOT);
        pub const INSERT: VirtKey = VirtKey(winuser::VK_INSERT);
        pub const DELETE: VirtKey = VirtKey(winuser::VK_DELETE);
        pub const HELP: VirtKey = VirtKey(winuser::VK_HELP);
        pub const LWIN: VirtKey = VirtKey(winuser::VK_LWIN);
        pub const RWIN: VirtKey = VirtKey(winuser::VK_RWIN);
        pub const APPS: VirtKey = VirtKey(winuser::VK_APPS);
        pub const SLEEP: VirtKey = VirtKey(winuser::VK_SLEEP);
        pub const NUMPAD0: VirtKey = VirtKey(winuser::VK_NUMPAD0);
        pub const NUMPAD1: VirtKey = VirtKey(winuser::VK_NUMPAD1);
        pub const NUMPAD2: VirtKey = VirtKey(winuser::VK_NUMPAD2);
        pub const NUMPAD3: VirtKey = VirtKey(winuser::VK_NUMPAD3);
        pub const NUMPAD4: VirtKey = VirtKey(winuser::VK_NUMPAD4);
        pub const NUMPAD5: VirtKey = VirtKey(winuser::VK_NUMPAD5);
        pub const NUMPAD6: VirtKey = VirtKey(winuser::VK_NUMPAD6);
        pub const NUMPAD7: VirtKey = VirtKey(winuser::VK_NUMPAD7);
        pub const NUMPAD8: VirtKey = VirtKey(winuser::VK_NUMPAD8);
        pub const NUMPAD9: VirtKey = VirtKey(winuser::VK_NUMPAD9);
        pub const MULTIPLY: VirtKey = VirtKey(winuser::VK_MULTIPLY);
        pub const ADD: VirtKey = VirtKey(winuser::VK_ADD);
        pub const SEPARATOR: VirtKey = VirtKey(winuser::VK_SEPARATOR);
        pub const SUBTRACT: VirtKey = VirtKey(winuser::VK_SUBTRACT);
        pub const DECIMAL: VirtKey = VirtKey(winuser::VK_DECIMAL);
        pub const DIVIDE: VirtKey = VirtKey(winuser::VK_DIVIDE);
        pub const F1: VirtKey = VirtKey(winuser::VK_F1);
        pub const F2: VirtKey = VirtKey(winuser::VK_F2);
        pub const F3: VirtKey = VirtKey(winuser::VK_F3);
        pub const F4: VirtKey = VirtKey(winuser::VK_F4);
        pub const F5: VirtKey = VirtKey(winuser::VK_F5);
        pub const F6: VirtKey = VirtKey(winuser::VK_F6);
        pub const F7: VirtKey = VirtKey(winuser::VK_F7);
        pub const F8: VirtKey = VirtKey(winuser::VK_F8);
        pub const F9: VirtKey = VirtKey(winuser::VK_F9);
        pub const F10: VirtKey = VirtKey(winuser::VK_F10);
        pub const F11: VirtKey = VirtKey(winuser::VK_F11);
        pub const F12: VirtKey = VirtKey(winuser::VK_F12);
        pub const F13: VirtKey = VirtKey(winuser::VK_F13);
        pub const F14: VirtKey = VirtKey(winuser::VK_F14);
        pub const F15: VirtKey = VirtKey(winuser::VK_F15);
        pub const F16: VirtKey = VirtKey(winuser::VK_F16);
        pub const F17: VirtKey = VirtKey(winuser::VK_F17);
        pub const F18: VirtKey = VirtKey(winuser::VK_F18);
        pub const F19: VirtKey = VirtKey(winuser::VK_F19);
        pub const F20: VirtKey = VirtKey(winuser::VK_F20);
        pub const F21: VirtKey = VirtKey(winuser::VK_F21);
        pub const F22: VirtKey = VirtKey(winuser::VK_F22);
        pub const F23: VirtKey = VirtKey(winuser::VK_F23);
        pub const F24: VirtKey = VirtKey(winuser::VK_F24);
        pub const NUMLOCK: VirtKey = VirtKey(winuser::VK_NUMLOCK);
        pub const SCROLL: VirtKey = VirtKey(winuser::VK_SCROLL);
        pub const OEM_NEC_EQUAL: VirtKey = VirtKey(winuser::VK_OEM_NEC_EQUAL);
        pub const OEM_FJ_JISHO: VirtKey = VirtKey(winuser::VK_OEM_FJ_JISHO);
        pub const OEM_FJ_MASSHOU: VirtKey = VirtKey(winuser::VK_OEM_FJ_MASSHOU);
        pub const OEM_FJ_TOUROKU: VirtKey = VirtKey(winuser::VK_OEM_FJ_TOUROKU);
        pub const OEM_FJ_LOYA: VirtKey = VirtKey(winuser::VK_OEM_FJ_LOYA);
        pub const OEM_FJ_ROYA: VirtKey = VirtKey(winuser::VK_OEM_FJ_ROYA);
        pub const LSHIFT: VirtKey = VirtKey(winuser::VK_LSHIFT);
        pub const RSHIFT: VirtKey = VirtKey(winuser::VK_RSHIFT);
        pub const LCONTROL: VirtKey = VirtKey(winuser::VK_LCONTROL);
        pub const RCONTROL: VirtKey = VirtKey(winuser::VK_RCONTROL);
        pub const LMENU: VirtKey = VirtKey(winuser::VK_LMENU);
        pub const RMENU: VirtKey = VirtKey(winuser::VK_RMENU);
        pub const BROWSER_BACK: VirtKey = VirtKey(winuser::VK_BROWSER_BACK);
        pub const BROWSER_FORWARD: VirtKey = VirtKey(winuser::VK_BROWSER_FORWARD);
        pub const BROWSER_REFRESH: VirtKey = VirtKey(winuser::VK_BROWSER_REFRESH);
        pub const BROWSER_STOP: VirtKey = VirtKey(winuser::VK_BROWSER_STOP);
        pub const BROWSER_SEARCH: VirtKey = VirtKey(winuser::VK_BROWSER_SEARCH);
        pub const BROWSER_FAVORITES: VirtKey =
            VirtKey(winuser::VK_BROWSER_FAVORITES);
        pub const BROWSER_HOME: VirtKey = VirtKey(winuser::VK_BROWSER_HOME);
        pub const VOLUME_MUTE: VirtKey = VirtKey(winuser::VK_VOLUME_MUTE);
        pub const VOLUME_DOWN: VirtKey = VirtKey(winuser::VK_VOLUME_DOWN);
        pub const VOLUME_UP: VirtKey = VirtKey(winuser::VK_VOLUME_UP);
        pub const MEDIA_NEXT_TRACK: VirtKey =
            VirtKey(winuser::VK_MEDIA_NEXT_TRACK);
        pub const MEDIA_PREV_TRACK: VirtKey =
            VirtKey(winuser::VK_MEDIA_PREV_TRACK);
        pub const MEDIA_STOP: VirtKey = VirtKey(winuser::VK_MEDIA_STOP);
        pub const MEDIA_PLAY_PAUSE: VirtKey =
            VirtKey(winuser::VK_MEDIA_PLAY_PAUSE);
        pub const LAUNCH_MAIL: VirtKey = VirtKey(winuser::VK_LAUNCH_MAIL);
        pub const LAUNCH_MEDIA_SELECT: VirtKey =
            VirtKey(winuser::VK_LAUNCH_MEDIA_SELECT);
        pub const LAUNCH_APP1: VirtKey = VirtKey(winuser::VK_LAUNCH_APP1);
        pub const LAUNCH_APP2: VirtKey = VirtKey(winuser::VK_LAUNCH_APP2);
        pub const OEM_1: VirtKey = VirtKey(winuser::VK_OEM_1);
        pub const OEM_PLUS: VirtKey = VirtKey(winuser::VK_OEM_PLUS);
        pub const OEM_COMMA: VirtKey = VirtKey(winuser::VK_OEM_COMMA);
        pub const OEM_MINUS: VirtKey = VirtKey(winuser::VK_OEM_MINUS);
        pub const OEM_PERIOD: VirtKey = VirtKey(winuser::VK_OEM_PERIOD);
        pub const OEM_2: VirtKey = VirtKey(winuser::VK_OEM_2);
        pub const OEM_3: VirtKey = VirtKey(winuser::VK_OEM_3);
        pub const OEM_4: VirtKey = VirtKey(winuser::VK_OEM_4);
        pub const OEM_5: VirtKey = VirtKey(winuser::VK_OEM_5);
        pub const OEM_6: VirtKey = VirtKey(winuser::VK_OEM_6);
        pub const OEM_7: VirtKey = VirtKey(winuser::VK_OEM_7);
        pub const OEM_8: VirtKey = VirtKey(winuser::VK_OEM_8);
        pub const OEM_AX: VirtKey = VirtKey(winuser::VK_OEM_AX);
        pub const OEM_102: VirtKey = VirtKey(winuser::VK_OEM_102);
        pub const ICO_HELP: VirtKey = VirtKey(winuser::VK_ICO_HELP);
        pub const ICO_00: VirtKey = VirtKey(winuser::VK_ICO_00);
        pub const PROCESSKEY: VirtKey = VirtKey(winuser::VK_PROCESSKEY);
        pub const ICO_CLEAR: VirtKey = VirtKey(winuser::VK_ICO_CLEAR);
        pub const PACKET: VirtKey = VirtKey(winuser::VK_PACKET);
        pub const OEM_RESET: VirtKey = VirtKey(winuser::VK_OEM_RESET);
        pub const OEM_JUMP: VirtKey = VirtKey(winuser::VK_OEM_JUMP);
        pub const OEM_PA1: VirtKey = VirtKey(winuser::VK_OEM_PA1);
        pub const OEM_PA2: VirtKey = VirtKey(winuser::VK_OEM_PA2);
        pub const OEM_PA3: VirtKey = VirtKey(winuser::VK_OEM_PA3);
        pub const OEM_WSCTRL: VirtKey = VirtKey(winuser::VK_OEM_WSCTRL);
        pub const OEM_CUSEL: VirtKey = VirtKey(winuser::VK_OEM_CUSEL);
        pub const OEM_ATTN: VirtKey = VirtKey(winuser::VK_OEM_ATTN);
        pub const OEM_FINISH: VirtKey = VirtKey(winuser::VK_OEM_FINISH);
        pub const OEM_COPY: VirtKey = VirtKey(winuser::VK_OEM_COPY);
        pub const OEM_AUTO: VirtKey = VirtKey(winuser::VK_OEM_AUTO);
        pub const OEM_ENLW: VirtKey = VirtKey(winuser::VK_OEM_ENLW);
        pub const OEM_BACKTAB: VirtKey = VirtKey(winuser::VK_OEM_BACKTAB);
        pub const ATTN: VirtKey = VirtKey(winuser::VK_ATTN);
        pub const CRSEL: VirtKey = VirtKey(winuser::VK_CRSEL);
        pub const EXSEL: VirtKey = VirtKey(winuser::VK_EXSEL);
        pub const EREOF: VirtKey = VirtKey(winuser::VK_EREOF);
        pub const PLAY: VirtKey = VirtKey(winuser::VK_PLAY);
        pub const ZOOM: VirtKey = VirtKey(winuser::VK_ZOOM);
        pub const NONAME: VirtKey = VirtKey(winuser::VK_NONAME);
        pub const PA1: VirtKey = VirtKey(winuser::VK_PA1);
        pub const OEM_CLEAR: VirtKey = VirtKey(winuser::VK_OEM_CLEAR);
    }

    impl fmt::Display for VirtKey {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self.0 {
                c => {
                    write!(f, "{}", c).map_err(|_| fmt::Error)?;
                }
            }
            Ok(())
        }
    }

    #[derive(Clone, Copy)]
    pub enum Modifier {
        None,
        Ctrl,
        Alt,
        Shift,
        CtrlAlt,
        CtrlShift,
        AltShift,
        CtrlAltShift,
    }

    impl fmt::Display for Modifier {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let v = match self {
                Modifier::None => "",
                Modifier::Ctrl => ", CONTROL",
                Modifier::Alt => ", ALT",
                Modifier::Shift => ", SHIFT",
                Modifier::CtrlAlt => ", CONTROL, ALT",
                Modifier::CtrlShift => ", CONTROL, SHIFT",
                Modifier::AltShift => ", ALT, SHIFT",
                Modifier::CtrlAltShift => ", CONTROL, ALT, SHIFT",
            };
            write!(f, "{}", v).map_err(|_| fmt::Error)?;
            Ok(())
        }
    }

    #[derive(Clone, Copy)]
    pub enum ASCIIModifier {
        None,
        Ctrl,
        Alt,
        CtrlAlt,
    }

    impl fmt::Display for ASCIIModifier {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let v = match self {
                ASCIIModifier::None => "",
                ASCIIModifier::Ctrl => ", CONTROL",
                ASCIIModifier::Alt => ", ALT",
                ASCIIModifier::CtrlAlt => ", CONTROL, ALT",
            };
            write!(f, "{}", v).map_err(|_| fmt::Error)?;
            Ok(())
        }
    }

    #[derive(Clone, Copy)]
    enum Key {
        ASCII {
            ascii_key: ASCIIKey,
            modifier: ASCIIModifier,
        },
        VirtKey {
            virt_key: VirtKey,
            modifier: Modifier,
        },
    }

    #[derive(Clone, Copy)]
    pub struct Event {
        key: Key,
        noinvert: bool,
    }

    impl Event {
        pub const fn virt_key_event(virt_key: VirtKey, modifier: Modifier) -> Self {
            Event {
                key: Key::VirtKey { virt_key, modifier },
                noinvert: false,
            }
        }

        pub const fn ascii_key_event(ascii_key: ASCIIKey, modifier: ASCIIModifier) -> Self {
            Event {
                key: Key::ASCII {
                    ascii_key,
                    modifier,
                },
                noinvert: false,
            }
        }

        #[deprecated]
        pub const fn noinvert(mut self) -> Self {
            self.noinvert = true;
            self
        }
    }

    #[derive(Default)]
    struct AcceleratorsItems {
        extra_info: Option<ExtraInfo>,
        events: Vec<(Id, Event)>,
    }

    #[derive(Default)]
    pub(crate) struct AcceleratorsData(OptionLangSpecific<AcceleratorsItems>);

    pub struct AcceleratorsBuilder(AcceleratorsData);

    builder_implement_priv_default!(AcceleratorsBuilder);
    builder_extra_info_methods!(AcceleratorsBuilder);
    builder_build_method!(AcceleratorsBuilder, crate::resource::Accelerators);

    impl AcceleratorsBuilder {
        pub fn event(mut self, id: impl Into<Id>, event: Event) -> Self {
            let id = id.into();
            let common_items = (self.0).0.access_fallback_mut();
            common_items.events.push((id, event));
            self
        }

        pub fn lang_specific_event(mut self, lang: Lang, id: impl Into<Id>, event: Event) -> Self {
            let id = id.into();
            let lang_items = (self.0).0.access_lang_specific_mut(lang);
            lang_items.events.push((id, event));
            self
        }
    }

    impl AcceleratorsData {
        pub(crate) fn is_missing_for_lang(&self, l: crate::Lang) -> bool {
            self.0.get(l).is_none()
        }

        pub(crate) fn write_resource_header_extras(&self, w: &mut dyn std::io::Write, l: crate::Lang) -> Result<(), std::io::Error> {
            let items = self.0.get(l).expect("unreachable!");
            crate::codegen::write_extra_info(w, &items.extra_info)?;
            Ok(())
        }

        pub(crate) fn write_resource_segment(&self, w: &mut dyn std::io::Write, l: crate::Lang) -> Result<(), std::io::Error> {
            let items = self.0.get(l).expect("unreachable!");
            write!(w, "{{\n")?;
            for (id, event) in items.events.iter() {
                let noinvert = if event.noinvert { ", NOINVERT" } else { "" };
                match event.key {
                    Key::ASCII{ascii_key, modifier} => {
                        write!(w, "\t{}, {}, ASCII{}{}\n", ascii_key, id, modifier, noinvert)?;
                    },
                    Key::VirtKey{virt_key, modifier} => {
                        write!(w, "\t{}, {}, VIRTKEY{}{}\n", virt_key, id, modifier, noinvert)?;
                    }
                }
            }
            write!(w, "}}\n")?;
            Ok(())
        }
    }
}

pub mod menu {
    use crate::{CowStr, Id, OptionLangSpecific};
    use crate::MultiLangText;
    use winapi::ctypes::c_int;
    use winapi::shared::minwindef::UINT;
    use winapi::um::winuser;

    #[derive(Clone, Copy, Default, PartialEq)]
    pub struct MenuType(UINT);

    impl MenuType {
        // pub const STRING: MenuType = MenuType(winuser::MFT_STRING); //zero, not needed.
        pub const BITMAP: MenuType = MenuType(winuser::MFT_BITMAP);
        pub const MENUBAR_BREAK: MenuType = MenuType(winuser::MFT_MENUBARBREAK);
        pub const MENU_BREAK: MenuType = MenuType(winuser::MFT_MENUBREAK);
        pub const OWNER_DRAW: MenuType = MenuType(winuser::MFT_OWNERDRAW);
        pub const RADIO_CHECK: MenuType = MenuType(winuser::MFT_RADIOCHECK);
        pub const SEPARATOR: MenuType = MenuType(winuser::MFT_SEPARATOR);
        pub const RIGHT_ORDER: MenuType = MenuType(winuser::MFT_RIGHTORDER);
        pub const RIGHT_JUSTIFY: MenuType = MenuType(winuser::MFT_RIGHTJUSTIFY);
    }

    bitflags_bitor_method!(MenuType);

    #[derive(Clone, Copy, Default, PartialEq)]
    pub struct MenuState(UINT);

    impl MenuState {
        // pub const GRAYED: MenuState = MenuState(winuser::MFS_GRAYED); // alias of DISABLED
        pub const DISABLED: MenuState = MenuState(winuser::MFS_DISABLED);
        pub const CHECKED: MenuState = MenuState(winuser::MFS_CHECKED);
        pub const HIGHLIGHTED: MenuState = MenuState(winuser::MFS_HILITE);
        // pub const ENABLED: MenuState = MenuState(winuser::MFS_ENABLED); // zero, not needed
        // pub const UNCHECKED: MenuState = MenuState(winuser::MFS_UNCHECKED); // zero, not needed
        // pub const UNHILITE: MenuState = MenuState(winuser::MFS_UNHILITE); // zero, not needed
        pub const DEFAULT_ITEM: MenuState = MenuState(winuser::MFS_DEFAULT);
    }

    bitflags_bitor_method!(MenuState);

    #[derive(Default)]
    struct PopupData {
        help_id: Option<c_int>,
        items: Vec<MenuItem>,
    }

    struct MenuItem {
        id: Option<Id>,
        text: OptionLangSpecific<CowStr>,
        ty: MenuType,
        state: MenuState,
        popup: Option<PopupData>,
    }

    #[derive(Default)]
    pub(crate) struct MenuData(Vec<MenuItem>);

    pub struct MenuBuilder(MenuData);

    builder_implement_priv_default!(MenuBuilder);
    builder_build_method!(MenuBuilder, crate::resource::Menu);

    impl MenuBuilder {
        fn internal_add_item(&mut self, id: Option<Id>, text: MultiLangText, ty: MenuType, state: MenuState, popup: Option<PopupData>) {
            (self.0).0.push(MenuItem {
                id,
                text: text.0,
                ty,
                state,
                popup
            });
        }
    }

    pub struct PopupBuilder(PopupData);
    builder_implement_priv_default!(PopupBuilder);

    impl PopupBuilder {
        pub fn help_id(mut self, help_id: c_int) -> Self {
            (self.0).help_id = Some(help_id);
            self
        }
        fn internal_add_item(&mut self, id: Option<Id>, text: MultiLangText, ty: MenuType, state: MenuState, popup: Option<PopupData>) {
            (self.0).items.push(MenuItem {
                id,
                text: text.0,
                ty,
                state,
                popup
            });
        }
    }

    macro_rules! declare_menu_append_operations {
        ($builder_ty:ident) => {
            impl $builder_ty {
                pub fn popup(mut self, text: impl Into<MultiLangText>, popup_building: impl FnOnce(PopupBuilder) -> PopupBuilder) -> Self {
                    let popup_builder = popup_building(<PopupBuilder as crate::PrivDefault>::priv_default());
                    self.internal_add_item(None, text.into(), MenuType::default(), MenuState::default(), Some(popup_builder.0));
                    self
                }
                pub fn item(mut self, id: impl Into<Id>, text: impl Into<MultiLangText>) -> Self {
                    self.internal_add_item(Some(id.into()), text.into(), 
                        MenuType::default(), MenuState::default(), None);
                    self
                }
                pub fn separator(mut self) -> Self {
                    self.internal_add_item(None, MultiLangText::empty(), 
                        MenuType::SEPARATOR, MenuState::default(), None);
                    self
                }

                pub fn complex_popup(mut self, id: Option<impl Into<Id>>, text: impl Into<MultiLangText>, ty: MenuType, state: MenuState, popup_building: impl FnOnce(PopupBuilder) -> PopupBuilder) -> Self {
                    let popup_builder = popup_building(<PopupBuilder as crate::PrivDefault>::priv_default());
                    self.internal_add_item(id.map(Into::into), text.into(), 
                        ty, state, Some(popup_builder.0));
                    self
                }

                pub fn complex_item(mut self, id: Option<impl Into<Id>>, text: impl Into<MultiLangText>, ty: MenuType, state: MenuState) -> Self {
                    self.internal_add_item(id.map(Into::into), text.into(), 
                        ty, state, None);
                    self
                }
            }
        };
    }

    declare_menu_append_operations!(MenuBuilder);
    declare_menu_append_operations!(PopupBuilder);

    use std::io::Error as IOError;

    impl MenuData {
        pub(crate) fn is_missing_for_lang(&self, lang: crate::Lang) -> bool {
            for item in self.0.iter() {
                if item.text.get(lang).is_some() {
                    return false;
                }
            }
            true
        }

        pub(crate) fn write_resource_header_extras(&self, w: &mut dyn std::io::Write, l: crate::Lang) -> Result<(), IOError> {
            Ok(())
        }

        fn write_menu_item_resouce_segment(w: &mut dyn std::io::Write, lang: crate::Lang, item: &MenuItem, indent: usize) -> Result<(), IOError> {
            let text = if let Some(text) = item.text.get(lang) {
                text
            } else {
                return Ok(());
            };
            for _ in 0..indent {
                write!(w, "\t")?;
            }
            let is_popup = item.popup.is_some(); 
            let kind = if is_popup { "POPUP" } else { "MENUITEM" };
            write!(w, "{} ", kind)?;
            crate::codegen::write_narrow_str(w, text)?;
            let exist_id = item.id.is_some();
            let exist_ty = item.ty != MenuType::default();
            let exist_state = item.state != MenuState::default();
            let exist_help_id = item.popup.as_ref().map(|popup| popup.help_id.is_some()).unwrap_or(false);
            if exist_id || exist_ty || exist_state || exist_help_id {
                write!(w, ", ")?;
            }
            if exist_id {
                let id = item.id.clone().unwrap();
                write!(w, "{}", id)?;
            }
            if exist_ty || exist_state || exist_help_id {
                write!(w, ", ")?;
            }
            if exist_ty {
                crate::codegen::write_dword(w, item.ty.0)?;
            }
            if exist_state || exist_help_id {
                write!(w, ", ")?;
            }
            if exist_state {
                crate::codegen::write_dword(w, item.state.0)?;
            }
            if exist_help_id {
                write!(w, ", ")?;
            }
            if exist_help_id {
                crate::codegen::write_c_int(w, item.popup.as_ref().unwrap().help_id.clone().unwrap())?;
            }
            write!(w, "\n")?;
            if is_popup {
                for _ in 0..indent {
                    write!(w, "\t")?;
                }
                write!(w, "{{\n")?;
                let inner_indent = indent + 1;
                for inner_item in item.popup.as_ref().unwrap().items.iter() {
                    Self::write_menu_item_resouce_segment(w, lang, inner_item, inner_indent)?;
                }
                for _ in 0..indent {
                    write!(w, "\t")?;
                }
                write!(w, "}}\n")?;
            }
            Ok(())            
        }

        pub(crate) fn write_resource_segment(&self, w: &mut dyn std::io::Write, l: crate::Lang) -> Result<(), IOError> {
            write!(w, "{{\n")?;
            for item in self.0.iter() {
                Self::write_menu_item_resouce_segment(w, l, item, 1)?;
            }
            write!(w, "}}\n")?;
            Ok(())
        }
    }
}

pub mod dialog {
    use crate::{CowStr, ExtraInfo, Id, IdOrName, OptionLangSpecific};
    use winapi::ctypes::c_int;
    use winapi::ctypes::c_long;
    use winapi::shared::minwindef::TRUE;
    use winapi::shared::minwindef::{BOOL, BYTE, DWORD};
    use winapi::um::wingdi;

    pub struct Rect {
        x: c_int,
        y: c_int,
        width: c_int,
        height: c_int,
    }

    pub struct Font {
        pointsize: c_int,
        typeface: CowStr,
        weight: FontWeight,
        italic: FontItalic,
        charset: FontCharset,
    }

    #[derive(Default)]
    pub struct FontWeight(c_long);

    impl FontWeight {
        // pub const DONTCARE: FontWeight = FontWeight(wingdi::FW_DONTCARE);  //zero, use FontWeight::default()
        pub const THIN: FontWeight = FontWeight(wingdi::FW_THIN);
        pub const EXTRALIGHT: FontWeight = FontWeight(wingdi::FW_EXTRALIGHT);
        pub const LIGHT: FontWeight = FontWeight(wingdi::FW_LIGHT);
        pub const NORMAL: FontWeight = FontWeight(wingdi::FW_NORMAL);
        pub const MEDIUM: FontWeight = FontWeight(wingdi::FW_MEDIUM);
        pub const SEMIBOLD: FontWeight = FontWeight(wingdi::FW_SEMIBOLD);
        pub const BOLD: FontWeight = FontWeight(wingdi::FW_BOLD);
        pub const EXTRABOLD: FontWeight = FontWeight(wingdi::FW_EXTRABOLD);
        pub const HEAVY: FontWeight = FontWeight(wingdi::FW_HEAVY);
        // pub const ULTRALIGHT: FontWeight = FontWeight(wingdi::FW_ULTRALIGHT); // alias of EXTRALIGHT
        // pub const REGULAR: FontWeight = FontWeight(wingdi::FW_REGULAR); // alias of NORMAL
        // pub const DEMIBOLD: FontWeight = FontWeight(wingdi::FW_DEMIBOLD); // alias of SEMIBOLD
        // pub const ULTRABOLD: FontWeight = FontWeight(wingdi::FW_ULTRABOLD); // alias of EXTRABOLD
        // pub const BLACK: FontWeight = FontWeight(wingdi::FW_BLACK); // alias of HEAVY
    }

    #[derive(Default)]
    pub struct FontItalic(BOOL);

    impl FontItalic {
        // const NORMAL: FontItalic = FontItalic(FALSE); // zero, use FontItalic::default()
        const ITALIC: FontItalic = FontItalic(TRUE);
    }

    pub struct FontCharset(BYTE);

    impl Default for FontCharset {
        fn default() -> Self {
            FontCharset(wingdi::DEFAULT_CHARSET as _)
        }
    }

    impl FontCharset {
        //pub const DEFAULT: FontCharset = FontCharset(wingdi::DEFAULT_CHARSET as _); // default, use FontCharset::default()
        pub const ANSI: FontCharset = FontCharset(wingdi::ANSI_CHARSET as _);
        pub const OEM: FontCharset = FontCharset(wingdi::OEM_CHARSET as _);
        pub const MAC: FontCharset = FontCharset(wingdi::MAC_CHARSET as _);
        pub const SYMBOL: FontCharset = FontCharset(wingdi::SYMBOL_CHARSET as _);
        pub const SHIFTJIS: FontCharset = FontCharset(wingdi::SHIFTJIS_CHARSET as _);
        pub const HANGEUL: FontCharset = FontCharset(wingdi::HANGEUL_CHARSET as _);
        pub const HANGUL: FontCharset = FontCharset(wingdi::HANGUL_CHARSET as _);
        pub const GB2312: FontCharset = FontCharset(wingdi::GB2312_CHARSET as _);
        pub const CHINESEBIG5: FontCharset = FontCharset(wingdi::CHINESEBIG5_CHARSET as _);
        pub const JOHAB: FontCharset = FontCharset(wingdi::JOHAB_CHARSET as _);
        pub const HEBREW: FontCharset = FontCharset(wingdi::HEBREW_CHARSET as _);
        pub const ARABIC: FontCharset = FontCharset(wingdi::ARABIC_CHARSET as _);
        pub const GREEK: FontCharset = FontCharset(wingdi::GREEK_CHARSET as _);
        pub const TURKISH: FontCharset = FontCharset(wingdi::TURKISH_CHARSET as _);
        pub const VIETNAMESE: FontCharset = FontCharset(wingdi::VIETNAMESE_CHARSET as _);
        pub const THAI: FontCharset = FontCharset(wingdi::THAI_CHARSET as _);
        pub const EASTEUROPE: FontCharset = FontCharset(wingdi::EASTEUROPE_CHARSET as _);
        pub const RUSSIAN: FontCharset = FontCharset(wingdi::RUSSIAN_CHARSET as _);
        pub const BALTIC: FontCharset = FontCharset(wingdi::BALTIC_CHARSET as _);
    }

    struct DialogControlStyle(Option<DWORD>, Option<DWORD>);

    pub struct DialogControl {
        template: ControlTemplate,
        id: Id,
        text: OptionLangSpecific<CowStr>,
        rect: Rect,
        class: Option<CowStr>,
        style: Option<DialogControlStyle>,
    }

    pub struct ControlTemplate {
        name: &'static str,
        use_text: bool,
        use_size: bool,
        use_keyword: Option<&'static str>,
    }

    impl ControlTemplate {
        pub const CONTROL: ControlTemplate = ControlTemplate {
            name: "CONTROL",
            use_text: true,
            use_size: true,
            use_keyword: None,
        };
        pub const AUTO3STATE: ControlTemplate = ControlTemplate {
            name: "AUTO3STATE",
            use_text: true,
            use_size: true,
            use_keyword: Some("BUTTON"),
        };
        pub const AUTOCHECKBOX: ControlTemplate = ControlTemplate {
            name: "AUTOCHECKBOX",
            use_text: true,
            use_size: true,
            use_keyword: Some("BUTTON"),
        };
        pub const AUTORADIOBUTTON: ControlTemplate = ControlTemplate {
            name: "AUTORADIOBUTTON",
            use_text: true,
            use_size: true,
            use_keyword: Some("BUTTON"),
        };
        pub const CHECKBOX: ControlTemplate = ControlTemplate {
            name: "CHECKBOX",
            use_text: true,
            use_size: true,
            use_keyword: Some("BUTTON"),
        };
        pub const COMBOBOX: ControlTemplate = ControlTemplate {
            name: "COMBOBOX",
            use_text: false,
            use_size: true,
            use_keyword: Some("COMBOBOX"),
        };
        pub const CTEXT: ControlTemplate = ControlTemplate {
            name: "CTEXT",
            use_text: true,
            use_size: true,
            use_keyword: Some("STATIC"),
        };
        pub const DEFPUSHBUTTON: ControlTemplate = ControlTemplate {
            name: "DEFPUSHBUTTON",
            use_text: true,
            use_size: true,
            use_keyword: Some("BUTTON"),
        };
        pub const EDITTEXT: ControlTemplate = ControlTemplate {
            name: "EDITTEXT",
            use_text: true,
            use_size: true,
            use_keyword: Some("EDIT"),
        };
        pub const GROUPBOX: ControlTemplate = ControlTemplate {
            name: "GROUPBOX",
            use_text: true,
            use_size: true,
            use_keyword: Some("BUTTON"),
        };
        pub const ICON: ControlTemplate = ControlTemplate {
            name: "ICON",
            use_text: true,
            use_size: false,
            use_keyword: Some("STATIC"),
        };
        pub const LISTBOX: ControlTemplate = ControlTemplate {
            name: "LISTBOX",
            use_text: false,
            use_size: true,
            use_keyword: Some("LISTBOX"),
        };
        pub const LTEXT: ControlTemplate = ControlTemplate {
            name: "LTEXT",
            use_text: true,
            use_size: true,
            use_keyword: Some("STATIC"),
        };
        pub const PUSHBOX: ControlTemplate = ControlTemplate {
            name: "PUSHBOX",
            use_text: true,
            use_size: true,
            use_keyword: Some("BUTTON"),
        };
        pub const PUSHBUTTON: ControlTemplate = ControlTemplate {
            name: "PUSHBUTTON",
            use_text: true,
            use_size: true,
            use_keyword: Some("BUTTON"),
        };
        pub const RADIOBUTTON: ControlTemplate = ControlTemplate {
            name: "RADIOBUTTON",
            use_text: true,
            use_size: true,
            use_keyword: Some("BUTTON"),
        };
        pub const RTEXT: ControlTemplate = ControlTemplate {
            name: "RTEXT",
            use_text: true,
            use_size: true,
            use_keyword: Some("STATIC"),
        };
        pub const SCROLLBAR: ControlTemplate = ControlTemplate {
            name: "SCROLLBAR",
            use_text: true,
            use_size: true,
            use_keyword: Some("SCROLLBAR"),
        };
        pub const STATE3: ControlTemplate = ControlTemplate {
            name: "STATE3",
            use_text: true,
            use_size: true,
            use_keyword: Some("BUTTON"),
        };
    }

    pub struct DialogStyle(DWORD, DWORD);

    #[derive(Default)]
    pub(crate) struct DialogData {
        rect: OptionLangSpecific<Rect>,
        help_id: OptionLangSpecific<c_int>,
        extra_info: OptionLangSpecific<ExtraInfo>,
        caption: OptionLangSpecific<CowStr>,
        class: Option<IdOrName>,
        style: Option<DialogStyle>,
        font: Option<Font>,
        menu: Option<IdOrName>,
        controls: Vec<DialogControl>,
    }

    pub struct DialogBuilder(DialogData);

    builder_implement_priv_default!(DialogBuilder);
    builder_build_method!(DialogBuilder, crate::resource::Dialog);

    unimplemented_resouce_data_write_segment!(DialogData);
}

pub mod version_info {
    use crate::CowStr;
    use crate::OptionLangSpecific;
    use winapi::shared::minwindef::{DWORD, WORD};

    pub struct Version([WORD; 4]);
    pub struct FileFlags(DWORD);
    pub struct FileOS(DWORD);
    pub struct FileType(DWORD);

    #[derive(Default)]
    pub(crate) struct VersionInfoData {
        fixed_file_version: Option<Version>,
        fixed_product_version: Option<Version>,
        fixed_file_flags: Option<FileFlags>,
        fixed_file_os: Option<FileOS>,
        fixed_file_type: Option<FileType>,
        product_name: OptionLangSpecific<CowStr>,
        product_version: OptionLangSpecific<CowStr>,
        file_description: OptionLangSpecific<CowStr>,
        file_version: OptionLangSpecific<CowStr>,
        internal_name: OptionLangSpecific<CowStr>,
        original_filename: OptionLangSpecific<CowStr>,
        company_name: OptionLangSpecific<CowStr>,
        legal_copyright: Option<OptionLangSpecific<CowStr>>,
        legal_trademarks: Option<OptionLangSpecific<CowStr>>,
        private_build: Option<OptionLangSpecific<CowStr>>,
        special_build: Option<OptionLangSpecific<CowStr>>,
        comments: Option<OptionLangSpecific<CowStr>>,
    }

    //we only support Unicode as charset here.

    pub struct VersionInfoBuilder(VersionInfoData);

    builder_implement_priv_default!(VersionInfoBuilder);
    builder_build_method!(VersionInfoBuilder, crate::resource::VersionInfo);
    unimplemented_resouce_data_write_segment!(VersionInfoData);
}

pub mod rc_inline {
    use crate::{ExtraInfo, OptionLangSpecific};
    use winapi::shared::minwindef::{DWORD, WORD};

    enum RcInlineItem {
        U16(WORD),
        U32(DWORD),
        Str(Vec<u8>),
        WStr(Vec<u16>),
    }

    #[derive(Default)]
    pub(crate) struct RcInlineData {
        extra_info: OptionLangSpecific<ExtraInfo>,
        items: OptionLangSpecific<Vec<RcInlineItem>>,
    }

    pub struct RcInlineBuilder(RcInlineData);
    builder_implement_priv_default!(RcInlineBuilder);
    builder_extra_info_methods2!(RcInlineBuilder);
    builder_build_method!(RcInlineBuilder, crate::resource::RcInline);
    unimplemented_resouce_data_write_segment!(RcInlineData);
}

pub mod user_defined {
    use crate::rc_inline::RcInlineData;
    use crate::CowPath;

    pub(crate) enum UserDefinedData {
        RcInline(RcInlineData),
        External(CowPath),
    }

    impl Default for UserDefinedData {
        fn default() -> Self {
            UserDefinedData::RcInline(Default::default())
        }
    }

    impl From<CowPath> for UserDefinedData {
        fn from(path: CowPath) -> Self {
            UserDefinedData::External(path)
        }
    }

    pub struct UserDefinedBuilder(UserDefinedData);
    builder_implement_priv_default!(UserDefinedBuilder);
    builder_build_method!(UserDefinedBuilder, crate::resource::UserDefined);
}

impl Build {
    pub fn generate_rc_file(self, path: &std::path::Path) -> Result<(), io::Error> {
        use std::fs::File;
        let mut file = File::create(path)?;
        codegen::write_header(&mut file)?;

        for (lang, resource_list) in self.resources {
            for (id_or_name, resource) in resource_list {
                resource.write_script_segment(&mut file, lang, id_or_name)?;
            }
        }

        Ok(())
    }

    pub fn compile_rc_file(path: &std::path::Path) -> Result<(), io::Error> {
        embed_resource::compile(path);
        Ok(())
    }

    pub fn compile(self) -> Result<(), io::Error> {
        use std::path::PathBuf;
        let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR variable is not set");
        let mut rc_file_path = PathBuf::from(out_dir);
        rc_file_path.push("resource.rc");
        self.generate_rc_file(&rc_file_path)?;
        println!("rerun-if-changed={}", rc_file_path.display());
        Self::compile_rc_file(&rc_file_path)?;

        Ok(())
    }
}

mod codegen {
    use std::io::{Error as IOError, Write};
    use crate::{Id, IdOrName};
    use crate::CowStr;
    use crate::resource;

    pub(crate) fn write_header(w: &mut dyn Write) -> Result<(), IOError> {
        write!(
            w,
            "// Resource script automatically generated by RESW-RS.\n"
        )?;
        write!(w, "// Do not edit this file manually.\n")?;
        write!(w, "\n")?;
        write!(w, "#pragma codepage(65001)\n")?;
        Ok(())
    }

    pub(crate) fn write_c_int(w: &mut dyn Write, c_int: winapi::ctypes::c_int) -> Result<(), IOError> {
        if std::mem::size_of_val(&c_int) > 2 {
            write!(w, "{}L", c_int)
        } else {
            write!(w, "{}", c_int)
        }
    }

    pub(crate) fn write_dword(w: &mut dyn Write, dword: winapi::shared::minwindef::DWORD) -> Result<(), IOError> {
        write!(w, "{}L", dword)
    }

    pub(crate) fn need_escape_narrow_byte(v: &u8) -> bool {
        match v {
            0..=31u8 => true,
            b'\\' => true,
            b'\"' => true,
            127u8 => true,
            _ => false,
        }
    }

    pub(crate) fn need_escape_wide_u16(v: &u16) -> bool {
        match v {
            0..=31u16 => true,
            92u16 /*b'\\'*/ => true,
            34u16 /*b'\"'*/ => true,
            127u16 => true,
            32u16..=126u16 => false,
            _ => true,
        }
    }


    pub(crate) fn write_narrow_str(w: &mut dyn Write, string: &CowStr) -> Result<(), IOError> {
        write!(w, "\"")?;
        let mut rest_string = string.as_bytes();
        while !rest_string.is_empty() {
            let seq = rest_string.split(need_escape_narrow_byte).next().expect("unreachable");
            if !seq.is_empty() {
                w.write_all(seq)?;
                rest_string = &rest_string[seq.len()..];
            } else {
                write!(w, "\\{:03o}", rest_string[0])?;
                rest_string = &rest_string[1..];
            }
        }
        write!(w, "\"")?;
        Ok(())
    }

    #[cfg(windows)]
    fn write_wide_os_str(w: &mut dyn Write, name: &std::ffi::OsStr) -> Result<(), IOError> {
        use std::os::windows::ffi::OsStrExt;
        write!(w, "L\"")?;
        for ch in name.encode_wide() {
            if !need_escape_wide_u16(&ch) {
                debug_assert!(ch <= std::u8::MAX as _);
                let ch: [u8; 1] = [ch as u8];
                w.write_all(&ch)?;
            } else {
                write!(w, "\\x{:04x}", ch)?;
            }
        }
        write!(w, "\"")?;
        Ok(())
    }

    fn write_id_or_name(w: &mut dyn Write, id_or_name: &IdOrName) -> Result<(), IOError> {
        match id_or_name {
            IdOrName::Id(id) => write!(w, "{}", id),
            IdOrName::Name(name) => write_narrow_str(w, name),
        }
    }

    fn write_path(w: &mut dyn Write, path: &std::path::Path) -> Result<(), IOError> {
        let os_str = path.as_os_str();
        write_wide_os_str(w, os_str)
    }
    
    fn ensure_id_or_name_ignorable(id_or_name: &IdOrName) {
        match id_or_name {
            &IdOrName::Id(Id(v)) => {
                if v == 0 || v == (-1 as _) {
                    return;
                }
            },
            IdOrName::Name(s) => {
                if s == "" || s == " " || s == "_" {
                    return;
                }
            },
        }
        eprintln!("Warning: Expected ignorable id or name, found {:?}. Ignored.", id_or_name);
    }

    pub(crate) fn write_extra_info(
        w: &mut dyn Write,
        extra_info: &Option<crate::ExtraInfo>
    ) -> Result<(), IOError> {
        if let Some(extra_info) = extra_info {
            if let Some(characteristics) = &extra_info.characteristics {
                write!(w, " ")?;
                write_dword(w, *characteristics)?;
            }
            if let Some(version) = &extra_info.version {
                write!(w, " ")?;
                write_dword(w, *version)?;
            }
        }
        Ok(())
    }

    pub(crate) fn write_path_only_resource(
        w: &mut dyn Write,
        lang: crate::Lang,
        id_or_name: crate::IdOrName,
        res_type_keyword: &'static str,
        path: &std::path::Path,
    ) -> Result<(), IOError> {
        write_resource_header(w, lang, id_or_name, res_type_keyword)?;
        write!(w, " ")?;
        let mut absolute_path = std::env::current_dir()?;
        absolute_path.push(path);
        write_path(w, &absolute_path)?;
        write!(w, "\n")?;
        Ok(())
    }

    pub(crate) fn write_resource_header(
        w: &mut dyn Write,
        lang: crate::Lang,
        id_or_name: crate::IdOrName,
        res_type_keyword: &'static str,
    ) -> Result<(), IOError> {
        write!(w, "LANGUAGE 0x{:x}, 0x{:x}\n", lang.0, lang.1)?;
        match res_type_keyword {
            resource::StringTable::TYPE_KEYWORD => {
                ensure_id_or_name_ignorable(&id_or_name);
            },
            resource::VersionInfo::TYPE_KEYWORD => {
                if id_or_name != IdOrName::Id(Id(1)) {
                    ensure_id_or_name_ignorable(&id_or_name);
                }
                write!(w, "1 ")?;
            },
            _ => {
                write_id_or_name(w, &id_or_name)?;
                write!(w, " ")?;
            }
        }
        write!(w, "{} ", res_type_keyword)?;
        Ok(())
    }
}

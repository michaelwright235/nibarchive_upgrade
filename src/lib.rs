#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

use nibarchive::{NIBArchive, Object as NibObject, ValueVariant as NibValueVariant};
use plist::{Dictionary, Uid, Value};
pub use nibarchive;

pub(crate) const ARCHIVER: &str = "NSKeyedArchiver";
pub(crate) const ARCHIVER_VERSION: u64 = 100000;

pub(crate) const ARCHIVER_KEY_NAME: &str = "$archiver";
pub(crate) const TOP_KEY_NAME: &str = "$top";
pub(crate) const OBJECTS_KEY_NAME: &str = "$objects";
pub(crate) const VERSION_KEY_NAME: &str = "$version";
pub(crate) const NULL_OBJECT_REFERENCE: &str = "$null";

pub(crate) const OBJECT_CLASS_KEY: &str = "$class";

pub(crate) const CLASSES_KEY: &str = "$classes";
pub(crate) const CLASSNAME_KEY: &str = "$classname";

fn nibvalue_to_plistvalue(val: NibValueVariant) -> Value {
    match val {
        NibValueVariant::Int8(v) => Value::Integer(v.into()),
        NibValueVariant::Int16(v) => Value::Integer(v.into()),
        NibValueVariant::Int32(v) => Value::Integer(v.into()),
        NibValueVariant::Int64(v) => Value::Integer(v.into()),
        NibValueVariant::Bool(v) => Value::Boolean(v),
        NibValueVariant::Float(v) => Value::Real(v.into()),
        NibValueVariant::Double(v) => Value::Real(v),
        NibValueVariant::Data(v) => Value::Data(v),
        NibValueVariant::Nil => Value::String(NULL_OBJECT_REFERENCE.into()), // TODO: check
        NibValueVariant::ObjectRef(v) => Value::Uid(Uid::new(v as u64)),
    }
}

fn reconstruct_object(object: &NibObject, archive: &NIBArchive, add_class: bool) -> Dictionary {
    let mut dict = Dictionary::new();
    // Add $class key
    if add_class {
        // We add all classes object in the end of `$objects` array. To find the matching class
        // we just add the count of regular objects to the current class uid.
        let uid = (object.class_name_index() as u64) + (archive.objects().len() as u64);
        dict.insert(OBJECT_CLASS_KEY.into(), Value::Uid(Uid::new(uid)));
    }
    let values = object.values(archive.values());

    if values.is_empty() {
        return dict;
    }

    let is_inlined = values[0].key(archive.keys()) == "NSInlinedValue"
        && &NibValueVariant::Bool(true) == values[0].value();

    if is_inlined {
        return reconstruct_inlined_object(object, archive, dict);
    }

    // Regular object
    for value in values {
        let key = value.key(archive.keys()).clone();
        let inner = nibvalue_to_plistvalue(value.value().clone());
        dict.insert(key, inner);
    }
    dict
}

/// NSArray, NSSet and NSDictionary (and their mutable versions) are inlined
/// ([more info](https://www.mothersruin.com/software/Archaeology/reverse/uinib.html#collections)).
/// So we bring back their normal structure.
fn reconstruct_inlined_object(object: &NibObject, archive: &NIBArchive, mut dict: Dictionary) -> Dictionary {
    let values = object.values(archive.values());
    let class_name = object.class_name(archive.class_names()).name();

    if class_name == "NSArray"
        || class_name == "NSMutableArray"
        || class_name == "NSSet"
        || class_name == "NSMutableSet"
    {
        let mut array = Vec::with_capacity(values.len() - 1);
        for value in &values[1..] {
            array.push(nibvalue_to_plistvalue(value.value().clone()));
        }
        dict.insert("NS.objects".into(), Value::Array(array));
    }
    else if class_name == "NSDictionary" || class_name == "NSMutableDictionary" {
        let mut dict_keys = Vec::with_capacity(values.len() / 2);
        let mut dict_values = Vec::with_capacity(values.len() / 2);
        let mut is_key = true;
        for value in &values[1..] {
            if is_key {
                dict_keys.push(nibvalue_to_plistvalue(value.value().clone()));
            } else {
                dict_values.push(nibvalue_to_plistvalue(value.value().clone()));
            }
            is_key = !is_key;
        }
        dict.insert("NS.keys".into(), Value::Array(dict_keys));
        dict.insert("NS.values".into(), Value::Array(dict_values));
    }
    else {
        println!("Unknown inlined object: {class_name}. The resulting file may be malformed.");
        for value in values {
            let key = value.key(archive.keys()).clone();
            let inner = nibvalue_to_plistvalue(value.value().clone());
            dict.insert(key, inner);
        }
    }
    dict
}

/// Converts a NIB Archive to a Cocoa Keyed Archive (NSKeyedArchive).
pub fn upgrade(archive: &NIBArchive) -> Value {
    let mut plist_root = Dictionary::new();
    // Add $archiver key
    plist_root.insert(ARCHIVER_KEY_NAME.into(), Value::String(ARCHIVER.into()));

    let objects = archive.objects();
    let class_names = archive.class_names();

    let mut plist_objects: Vec<Value> = Vec::with_capacity(objects.len());

    // It seems like all keyed archives has this as the first object
    plist_objects.push(Value::String(NULL_OBJECT_REFERENCE.into()));

    for object in objects.iter().skip(1) {
        // skip top object
        plist_objects.push(Value::Dictionary(reconstruct_object(object, archive, true)));
    }

    for class_name in class_names {
        let classes_array = vec![Value::String(class_name.name().into())];
        // $classes contains the main class as the first entry and then its parent.
        // I've seen only one example of fallback_classes: NSColor for UIColor object.
        // However NSColor is not a parent class of UIColor.
        /* for cls in class_name.fallback_classes(&class_names) {
            classes_array.push(cls.name().into());
        } */
        let mut class_obj = Dictionary::new();
        class_obj.insert(CLASSES_KEY.into(), Value::Array(classes_array));
        class_obj.insert(
            CLASSNAME_KEY.into(),
            Value::String(class_name.name().into()),
        );
        plist_objects.push(Value::Dictionary(class_obj));
    }

    // Add $objects key
    plist_root.insert(OBJECTS_KEY_NAME.into(), Value::Array(plist_objects));

    // Add $top key
    let top = reconstruct_object(&objects[0], archive, false);
    plist_root.insert(TOP_KEY_NAME.into(), Value::Dictionary(top));

    // Add $version key
    plist_root.insert(
        VERSION_KEY_NAME.into(),
        Value::Integer(ARCHIVER_VERSION.into()),
    );

    Value::Dictionary(plist_root)
}

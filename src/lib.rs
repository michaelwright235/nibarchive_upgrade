use nibarchive::{ClassName as NibClassName, NibArchive, Object as NibObject, Value as NibValue, ValueVariant as NibValueVariant};
use plist::{Dictionary, Uid, Value};

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
        NibValueVariant::Nil => Value::String("$null".into()), // TODO: check
        NibValueVariant::ObjectRef(v) => Value::Uid(Uid::new(v as u64)),
    }
}

fn reconstruct_object(object: &NibObject, archive: &NibArchive, add_class: bool) -> Dictionary {
    let mut dict = Dictionary::new();
    if add_class {
        let uid = (object.class_name_index() as u64) + (archive.objects().len() as u64);
        dict.insert("$class".into(), Value::Uid(Uid::new(uid)));
    }
    let values = object.values(archive.values());
    for value in values {
        let key = value.key(archive.keys()).clone();
        let inner = nibvalue_to_plistvalue(value.value().clone());
        dict.insert(key, inner);
    }
    dict
}

pub fn downgrade(archive: NibArchive) -> Value {
    let mut plist_root = Dictionary::new();
    plist_root.insert("$archiver".into(), Value::String("NSKeyedArchiver".into()));

    let objects = archive.objects();
    let values = archive.values();
    let keys = archive.keys();
    let class_names = archive.class_names();

    let mut plist_classes: Vec<Value> = Vec::with_capacity(class_names.len());

    let mut plist_objects: Vec<Value> = Vec::with_capacity(objects.len());
    plist_objects.push(Value::String("$null".into()));
    for i in 1..objects.len() { // skip top object
        plist_objects.push(Value::Dictionary(reconstruct_object(&objects[i], &archive, true)));
    }

    for class_name in class_names {
        let mut classes_array = vec![Value::String(class_name.name().into())];
        for cls in class_name.fallback_classes(&class_names) {
            classes_array.push(cls.name().into()) // TODO: check
        }
        let mut class_obj = Dictionary::new();
        class_obj.insert("$classes".into(), Value::Array(classes_array));
        class_obj.insert("$classname".into(), Value::String(class_name.name().into()));
        plist_objects.push(Value::Dictionary(class_obj));
    }

    plist_root.insert("$objects".into(), Value::Array(plist_objects));

    let top = reconstruct_object(&objects[0], &archive, false);
    plist_root.insert("$top".into(), Value::Dictionary(top));

    plist_root.insert("$version".into(), Value::Integer(100000.into()));

    Value::Dictionary(plist_root)
}


#[cfg(test)]
mod tests {
    use nibarchive::NibArchive;

    use crate::downgrade;


    #[test]
    fn b() {
        let nib = NibArchive::from_file("./tests/View.nib").unwrap();
        let plist = downgrade(nib);
        plist::to_file_binary("./tests/View.plist", &plist).unwrap();
    }
}

use std::collections::BTreeMap;
use lopdf::{Bookmark, Document, ObjectId, Object};
use tauri::{AppHandle, Manager};
use tauri::ipc::Response;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
  format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn combine_pdf(file1: &str, file2: &str,app_handle: AppHandle) ->Response {

  let cache_path = app_handle.path().app_cache_dir().unwrap();
  let file_path1 = format!("{cache}/{val}", cache=cache_path.to_str().unwrap(), val=file1);
  let file_path2 = format!("{cache}/{val}", cache=cache_path.to_str().unwrap(), val=file2);
  let doc1 = Document::load(file_path1.clone()).unwrap();
  let doc2 = Document::load(file_path2.clone()).unwrap();
  let documents = vec![doc1,doc2];
  println!("{:?}", file_path1);
  let mut document = combine_docs(documents).unwrap();

  // Save the merged PDF.
  // Store file in current working directory.
  // Note: Line is excluded when running doc tests
  if true {
    document.save(format!("{cache}/merged.pdf",cache=cache_path.to_str().unwrap())).unwrap();
  }
  let mut doc_val:Vec<u8> = Vec::new();
  document.save_to(&mut doc_val).unwrap();
  tauri::ipc::Response::new(doc_val)

}

fn combine_docs(documents: Vec<Document>) -> Result<Document,()> {
  // Define a starting `max_id` (will be used as start index for object_ids).
  let mut max_id = 1;
  let mut pagenum = 1;
  // Collect all Documents Objects grouped by a map
  let mut documents_pages: BTreeMap<ObjectId, Object> = BTreeMap::new();
  let mut documents_objects: BTreeMap<ObjectId, Object> = BTreeMap::new();
  let mut document = Document::with_version("1.5");
  for mut doc in documents {
    let mut first = false;
    doc.renumber_objects_with(max_id);

    max_id = doc.max_id + 1;

    documents_pages.extend(
      doc
        .get_pages()
        .into_iter()
        .map(|(_, object_id)| {
          if !first {
            let bookmark = Bookmark::new(String::from(format!("Page_{}", pagenum)), [0.0, 0.0, 1.0], 0, object_id);
            document.add_bookmark(bookmark, None);
            first = true;
            pagenum += 1;
          }

          (
            object_id,
            doc.get_object(object_id).unwrap().to_owned(),
          )
        })
        .collect::<BTreeMap<ObjectId, Object>>(),
    );
    documents_objects.extend(doc.objects);
  }
  let mut catalog_object: Option<(ObjectId, Object)> = None;
  let mut pages_object: Option<(ObjectId, Object)> = None;

  // Process all objects except "Page" type
  for (object_id, object) in documents_objects.iter() {
    // We have to ignore "Page" (as are processed later), "Outlines" and "Outline" objects.
    // All other objects should be collected and inserted into the main Document.
    match object.type_name().unwrap_or(b"") {
      b"Catalog" => {
        // Collect a first "Catalog" object and use it for the future "Pages".
        catalog_object = Some((
          if let Some((id, _)) = catalog_object {
            id
          } else {
            *object_id
          },
          object.clone(),
        ));
      }
      b"Pages" => {
        // Collect and update a first "Pages" object and use it for the future "Catalog"
        // We have also to merge all dictionaries of the old and the new "Pages" object
        if let Ok(dictionary) = object.as_dict() {
          let mut dictionary = dictionary.clone();
          if let Some((_, ref object)) = pages_object {
            if let Ok(old_dictionary) = object.as_dict() {
              dictionary.extend(old_dictionary);
            }
          }

          pages_object = Some((
            if let Some((id, _)) = pages_object {
              id
            } else {
              *object_id
            },
            Object::Dictionary(dictionary),
          ));
        }
      }
      b"Page" => {}     // Ignored, processed later and separately
      b"Outlines" => {} // Ignored, not supported yet
      b"Outline" => {}  // Ignored, not supported yet
      _ => {
        document.objects.insert(*object_id, object.clone());
      }
    }
  }

  // If no "Pages" object found, abort.
  if pages_object.is_none() {
    println!("Pages root not found.");

    return Err(());
  }

  // Iterate over all "Page" objects and collect into the parent "Pages" created before
  for (object_id, object) in documents_pages.iter() {
    if let Ok(dictionary) = object.as_dict() {
      let mut dictionary = dictionary.clone();
      dictionary.set("Parent", pages_object.as_ref().unwrap().0);

      document
        .objects
        .insert(*object_id, Object::Dictionary(dictionary));
    }
  }

  // If no "Catalog" found, abort.
  if catalog_object.is_none() {
    println!("Catalog root not found.");

    return Err(());
  }

  let catalog_object = catalog_object.unwrap();
  let pages_object = pages_object.unwrap();

  // Build a new "Pages" with updated fields
  if let Ok(dictionary) = pages_object.1.as_dict() {
    let mut dictionary = dictionary.clone();

    // Set new pages count
    dictionary.set("Count", documents_pages.len() as u32);

    // Set new "Kids" list (collected from documents pages) for "Pages"
    dictionary.set(
      "Kids",
      documents_pages
        .into_iter()
        .map(|(object_id, _)| Object::Reference(object_id))
        .collect::<Vec<_>>(),
    );

    document
      .objects
      .insert(pages_object.0, Object::Dictionary(dictionary));
  }

  // Build a new "Catalog" with updated fields
  if let Ok(dictionary) = catalog_object.1.as_dict() {
    let mut dictionary = dictionary.clone();
    dictionary.set("Pages", pages_object.0);
    dictionary.remove(b"Outlines"); // Outlines not supported in merged PDFs

    document
      .objects
      .insert(catalog_object.0, Object::Dictionary(dictionary));
  }

  document.trailer.set("Root", catalog_object.0);

  // Update the max internal ID as wasn't updated before due to direct objects insertion
  document.max_id = document.objects.len() as u32;

  // Reorder all new Document objects
  document.renumber_objects();

  // Set any Bookmarks to the First child if they are not set to a page
  document.adjust_zero_pages();

  // // Set all bookmarks to the PDF Object tree then set the Outlines to the Bookmark content map.
  // if let Some(n) = document.build_outline() {
  //   if let Ok(Object::Dictionary(mut dict)) = document.get_object_mut(catalog_object.0) {
  //     dict.set("Outlines", Object::Reference(n));
  //   }
  // }

  document.compress();
  Ok(document)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_fs::init())
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_opener::init())
    .invoke_handler(tauri::generate_handler![greet])
    .invoke_handler(tauri::generate_handler![combine_pdf])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
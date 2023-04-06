use std::fs::File;
use std::io::Result as IOResult;
use std::path::Path;
use xml::reader::{EventReader, ParserConfig, XmlEvent};

//  Possible Errors ->
//      File Opening: NotFound, Permission Denied, InvalidInput

// Handling Every Error When Deserializing a XML File
pub fn read_xml_file<P: AsRef<Path>>(file_path: P) -> IOResult<String> {
    let source = File::open(file_path)?;

    let parser_config = ParserConfig {
        trim_whitespace: true,
        whitespace_to_characters: false,
        cdata_to_characters: true,
        ignore_comments: false,
        coalesce_characters: true,
        extra_entities: std::collections::HashMap::new(),
        ignore_end_of_stream: false,
        replace_unknown_entity_references: false,
        ignore_root_level_whitespace: true,
    };

    let chars_content = EventReader::new_with_config(source, parser_config);

    let mut content = String::new();

    // TODO: Handle Err Variant, Maybe???
    for data in chars_content {
        if let Ok(character_string_result) = data {
            if let XmlEvent::Characters(character_string) = character_string_result {
                content.push_str(&character_string);
                content.push_str(" ");
            }
        }
    }

    Ok(content)
}

// Documentation strings for built-in functions.

pub fn build_doc_strings(&self) {
    self.builtin_doc.insert(
        "+".to_owned(),
        ") j k -- j+k ) Push j+k on the stack".to_owned(),
    );
    self.builtin_doc.insert(
        "-".to_owned(),
        ") j k -- j-k ) Push  -k on the stack".to_owned(),
    );
    self.builtin_doc.insert(
        "*".to_owned(),
        ") j k -- j*k ) Push j*k on the stack".to_owned(),
    );
    self.builtin_doc.insert(
        "/".to_owned(),
        ") j k -- j/k ) Push j/k on the stack".to_owned(),
    );
    self.builtin_doc.insert(
        "%".to_owned(),
        ") j k -- j/k ) Push j%k on the stack".to_owned(),
    );
}

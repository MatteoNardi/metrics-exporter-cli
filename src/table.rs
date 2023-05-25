use std::fmt::Display;

#[derive(Debug)]
pub struct TableBuilder {
    header: Vec<Entry>,
}

impl TableBuilder {
    pub fn new() -> Self {
        Self { header: Vec::new() }
    }

    pub fn group(
        mut self,
        name: &str,
        mut f: impl FnMut(TableBuilder) -> TableBuilder,
    ) -> TableBuilder {
        self.header.push(Entry::Group(Group {
            name: name.to_string(),
            entries: f(TableBuilder::new()).header,
        }));
        self
    }

    pub fn field(mut self, name: &str, display_kind: DisplayKind) -> TableBuilder {
        self.header.push(Entry::Field(Field {
            name: name.to_string(),
            display: DisplayInfo {
                len: name.len(),
                align: if matches!(display_kind, DisplayKind::Histogram) {
                    Align::Left
                } else {
                    Align::Right
                },
                display_kind,
                margin_left: 0,
            },
            last_value: Value::Int(0),
            full_path: vec![],
        }));
        self
    }

    pub fn build(self) -> Table {
        let mut header_lines = Vec::new();
        let mut header = self.header;

        let depth = depth(&header);
        header_lines.resize_with(depth, Default::default);
        force_uniform_depth(&mut header, depth);
        compute_field_paths(&mut header, vec![]);
        fill_header_lines(&mut header, depth, &mut header_lines);
        header_lines
            .iter_mut()
            .for_each(|x| *x = x.trim_end().to_string());

        let mut fields = collect_fields(header);
        add_padding(&mut fields);

        Table {
            header_lines,
            fields,
        }
    }
}

/// Make sure all entries have the given depth by inserting empty groups
/// around entries.
fn force_uniform_depth(entries: &mut Vec<Entry>, expected_depth: usize) {
    for entry in entries.iter_mut() {
        let entry_depth = match entry {
            Entry::Group(group) => depth(&group.entries) + 1,
            Entry::Field(_) => 1,
        };
        if entry_depth < expected_depth {
            for _ in 0..(expected_depth - entry_depth) {
                *entry = Entry::Group(Group {
                    name: String::new(),
                    entries: vec![entry.clone()],
                });
            }
        }
    }
}

/// Fill the field full_path by traversing the tree
fn compute_field_paths(entries: &mut Vec<Entry>, path: Vec<String>) {
    for entry in entries.iter_mut() {
        match entry {
            Entry::Group(group) => {
                let mut path = path.clone();
                path.push(group.name.clone());
                compute_field_paths(&mut group.entries, path);
            }
            Entry::Field(field) => {
                field.full_path = path.clone();
                field.full_path.push(field.name.clone());
            }
        };
    }
}

fn collect_fields(entries: Vec<Entry>) -> Vec<Field> {
    let mut result = Vec::new();
    for entry in entries {
        match entry {
            Entry::Group(group) => result.append(&mut collect_fields(group.entries)),
            Entry::Field(field) => result.push(field),
        }
    }
    result
}

// Add to fields padding for the separator
fn add_padding(fields: &mut Vec<Field>) {
    for i in 1..fields.len() {
        let path1 = &fields[i - 1].full_path;
        let path2 = &fields[i].full_path;
        if path1.len() >= 2 || path2.len() >= 2 {
            let group1 = &path1[0..path1.len() - 1];
            let group2 = &path2[0..path2.len() - 1];
            if group1 != group2 {
                fields[i].display.margin_left = 2;
            }
        }
    }
}

fn depth(entries: &Vec<Entry>) -> usize {
    entries
        .iter()
        .map(|entry| match entry {
            Entry::Group(group) => depth(&group.entries) + 1,
            Entry::Field(_) => 1,
        })
        .max()
        .unwrap_or(0)
}

fn fill_header_lines(entries: &mut Vec<Entry>, depth: usize, mut lines: &mut Vec<String>) -> usize {
    let mut len = 0;
    let mut it = entries.iter_mut().peekable();
    while let Some(entry) = it.next() {
        match entry {
            Entry::Group(ref mut group) => {
                let i = lines.len() - depth;
                let mut child_len = fill_header_lines(&mut group.entries, depth - 1, &mut lines);
                // enlarge child to fit parent
                while child_len < group.name.len() {
                    for j in (i + 1)..lines.len() {
                        lines[j].push(' ');
                    }
                    child_len += 1;
                    let mut g: &mut Group = group;
                    loop {
                        match g.entries.last_mut().expect("empty group") {
                            Entry::Group(ref mut group) => g = group,
                            Entry::Field(last_field) => {
                                last_field.display.len += 1;
                                break;
                            }
                        }
                    }
                }
                len += add_centered_str(&mut lines[i], &group.name, child_len);

                // Join groups with " | "
                if depth != 1 && it.peek().is_some() {
                    for j in i..lines.len() {
                        lines[j].push_str(&" | ");
                    }
                    len += 3;
                }
            }
            Entry::Field(field) => {
                if depth == 1 {
                    lines.last_mut().unwrap().push_str(&field.name);
                    len += field.display.len;
                } else {
                    // Unreachable because of force_uniform_depth
                    unreachable!();
                }
            }
        }
        // Join terminal fields with " "
        if depth == 1 && it.peek().is_some() {
            lines.last_mut().unwrap().push(' ');
            len += 1;
        }
    }
    len
}

/// Append value to output, making sure it takes at least minimum_len characters.
/// Return the number of added characters. If minimum_len is bigger than value.len(),
/// text will be centered.
fn add_centered_str(output: &mut String, value: &str, minimum_len: usize) -> usize {
    let initial_len = output.len();
    if minimum_len > value.len() {
        let extra_space_to_insert = minimum_len - value.len();
        for _ in 0..((extra_space_to_insert) / 2) {
            output.push(' ');
        }
        output.push_str(value);
        for _ in 0..((extra_space_to_insert + 1) / 2) {
            output.push(' ');
        }
    } else {
        output.push_str(value);
    }
    output.len() - initial_len
}

pub struct Table {
    header_lines: Vec<String>,
    fields: Vec<Field>,
}

#[derive(Clone, Debug)]
enum Entry {
    Group(Group),
    Field(Field),
}

#[derive(Clone, Debug)]
struct Group {
    name: String,
    entries: Vec<Entry>,
}

#[derive(Clone, Debug)]
struct Field {
    name: String,
    full_path: Vec<String>,
    display: DisplayInfo,
    last_value: Value,
}

#[derive(Clone, Debug)]
struct DisplayInfo {
    /// How much space the field should take
    len: usize,
    margin_left: usize,
    align: Align,
    display_kind: DisplayKind,
}

#[derive(Clone, Debug)]
enum Align {
    Right,
    Left,
}

#[derive(Clone, Debug)]
pub enum DisplayKind {
    Number,
    Difference,
    Histogram,
}

#[derive(Clone, Debug)]
pub enum Value {
    Int(i64),
    F64(f64),
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::F64(value)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(x) => write!(f, "{}", x),
            Value::F64(x) => write!(f, "{}", x),
        }
    }
}

// positional?
impl Table {
    pub fn header(&self) -> String {
        self.header_lines.join("\n")
    }

    // Given a list of path components, with the last one being the field and
    // the first ones the gorups, return the entry position in the table, if found.
    pub fn position_of(&self, path: Vec<String>) -> Option<usize> {
        // ignore leading empty strings
        let items_to_ignore = if let Some(first_field) = self.fields.first() {
            first_field.full_path.len() - path.len()
        } else {
            0
        };
        self.fields
            .iter()
            .position(|field| field.full_path[items_to_ignore..] == path)
    }

    // Each entry gets an associated index at build time, field should be supplied in order
    pub fn display_row<T>(&mut self, values: Vec<T>) -> String
    where
        T: Into<Value>,
    {
        let mut output = String::new();
        for (value, mut field) in values.into_iter().zip(&mut self.fields) {
            for _ in 0..field.display.margin_left {
                output.push(' ');
            }
            let value = value.into();
            match field.display.display_kind {
                DisplayKind::Number => display_field(&mut output, field, value.to_string()),
                DisplayKind::Difference => {
                    let difference = match (&field.last_value, &value) {
                        (Value::Int(x), Value::Int(y)) => Value::Int(y - x),
                        (Value::F64(x), Value::F64(y)) => Value::F64(y - x),
                        (_, new_val) => new_val.clone(),
                    };
                    field.last_value = value;
                    display_field(&mut output, field, difference.to_string());
                }
                DisplayKind::Histogram => {
                    display_field(
                        &mut output,
                        field,
                        "#".repeat(match value {
                            Value::Int(x) => x as usize,
                            Value::F64(x) => x as usize,
                        }),
                    );
                }
            }
            output.push(' ');
        }
        output.pop();
        output
    }
}

const AUTOMATIC_GROWTH_MARGIN: usize = 1;

fn display_field(output: &mut String, field: &mut Field, v: String) {
    if field.display.len < v.len() {
        // When a table cell is asked to display a value too big for it's allocated space
        // (field.display.len), we'll automatically enlarge that cell to make it fit that
        // value.
        // To prevent too many size changes:
        // - the cell enlargement is permanent
        // - we add an extra AUTOMATIC_GROWTH_MARGIN space
        field.display.len = v.len() + AUTOMATIC_GROWTH_MARGIN;
    }
    if matches!(field.display.align, Align::Left) {
        output.push_str(&v);
    }
    for _ in 0..(field.display.len - v.len()) {
        output.push(' ')
    }
    if matches!(field.display.align, Align::Right) {
        output.push_str(&v);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table_a() -> Table {
        TableBuilder::new()
            .group("input", |input| {
                input
                    .field("counter", DisplayKind::Number)
                    .field("counter2", DisplayKind::Number)
            })
            .build()
    }

    #[test]
    fn header_simple() {
        let expected_l1 = "     input";
        let expected_l2 = "counter counter2";
        assert_eq!(table_a().header(), [expected_l1, expected_l2].join("\n"));
    }

    #[test]
    fn index_simple() {
        let table = table_a();

        let i_counter = table.position_of(vec!["input".to_string(), "counter".to_string()]);
        assert_eq!(i_counter, Some(0));
        let i_counter2 = table.position_of(vec!["input".to_string(), "counter2".to_string()]);
        assert_eq!(i_counter2, Some(1));
        let i_counter3 = table.position_of(vec!["input".to_string(), "counter3".to_string()]);
        assert_eq!(i_counter3, None);
    }

    #[test]
    fn value_simple() {
        let mut table = table_a();
        assert_eq!(&table.display_row(vec![2, 324]), "      2      324",);
    }

    #[test]
    fn value_enlargement() {
        let mut table = table_a();
        let actual = table.display_row(vec![2, 324]);
        let expected = "      2      324";
        assert_eq!(actual, expected);
        let actual = table.display_row(vec![11111111111, 324]);
        let expected = " 11111111111      324";
        assert_eq!(actual, expected);
        let actual = table.display_row(vec![111111111111, 324]);
        let expected = "111111111111      324";
        assert_eq!(actual, expected);
        let actual = table.display_row(vec![2, 324]);
        let expected = "           2      324";
        assert_eq!(actual, expected);
    }

    fn table_b() -> Table {
        TableBuilder::new()
            .group("g1", |input| {
                input
                    .field("c1", DisplayKind::Number)
                    .field("c2", DisplayKind::Number)
            })
            .group("g2", |input| {
                input
                    .field("c3", DisplayKind::Number)
                    .field("c4", DisplayKind::Number)
            })
            .build()
    }

    #[test]
    fn header_with_multiple_groups() {
        assert_eq!(
            table_b().header(),
            [
                //
                " g1   |  g2",
                "c1 c2 | c3 c4",
            ]
            .join("\n")
        );
    }

    #[test]
    fn index_with_multiple_groups() {
        let table = table_b();
        let actual = table.position_of(vec!["g1".to_string(), "c2".to_string()]);
        let expected = Some(1);
        assert_eq!(actual, expected);
        let actual = table.position_of(vec!["g2".to_string(), "c3".to_string()]);
        let expected = Some(2);
        assert_eq!(actual, expected);
    }

    #[test]
    fn table_with_blank_spots() {
        // Make sure group splitting keeps alignment when a column
        // (C in this example) has no super-group:
        let expected_l1 = "G1  |   | G2";
        let expected_l2 = "A B | C | D E";
        let expected_l3 = "1 2   3   4 5";

        let mut table = TableBuilder::new()
            .group("G1", |input| {
                input
                    .field("A", DisplayKind::Number)
                    .field("B", DisplayKind::Number)
            })
            .field("C", DisplayKind::Number)
            .group("G2", |input| {
                input
                    .field("D", DisplayKind::Number)
                    .field("E", DisplayKind::Number)
            })
            .build();
        let expected_header = [expected_l1, expected_l2].join("\n");
        assert_eq!(table.header(), expected_header);
        let actual = table.display_row(vec![1, 2, 3, 4, 5]);
        assert_eq!(actual, expected_l3);
    }

    #[test]
    fn table_with_large_group_label() {
        // Make sure column headers which are smaller than their parent
        // group, fill their space to align. In this example, "A" should
        // take 5 spaces, like "Large"
        // TODO: refactor fill_header_lines and align 1 with A
        let expected_l1 = "Large |";
        let expected_l2 = "A     | B";
        let expected_l3 = "    1   2";

        let mut table = TableBuilder::new()
            .group("Large", |input| input.field("A", DisplayKind::Number))
            .field("B", DisplayKind::Number)
            .build();
        let expected_header = [expected_l1, expected_l2].join("\n");
        assert_eq!(table.header(), expected_header);
        let actual = table.display_row(vec![1, 2]);
        assert_eq!(actual, expected_l3);
    }

    #[test]
    fn value_with_multiple_groups() {
        let mut table = table_b();
        assert_eq!(&table.display_row(vec![1, 2, 3, 4]), " 1  2    3  4");
    }

    #[test]
    fn value_difference() {
        let mut table = TableBuilder::new()
            .field("c1", DisplayKind::Difference)
            .build();
        assert_eq!(&table.display_row(vec![1]), " 1");
        assert_eq!(&table.display_row(vec![3]), " 2");
    }

    #[test]
    fn value_histogram() {
        let mut table = TableBuilder::new()
            .field("c1", DisplayKind::Histogram)
            .build();
        assert_eq!(&table.display_row(vec![1]), "# ");
        assert_eq!(&table.display_row(vec![3]), "### ");
        assert_eq!(&table.display_row(vec![1]), "#   ");
        assert_eq!(&table.display_row(vec![4]), "####");
    }
}

use egui::{Color32, ComboBox, DragValue, Painter, Sense, SidePanel, Stroke, UiBuilder, Vec2};
use taffy::{
    prelude::TaffyZero, AlignContent, AlignItems, AlignSelf, BoxSizing, Dimension, FlexDirection,
    LengthPercentage, LengthPercentageAuto, MaxTrackSizingFunction, MinTrackSizingFunction, NodeId,
    PrintTree, Size, Style, TaffyTree, TextAlign, TraversePartialTree,
};

#[derive(Default, Debug)]
pub struct TemplateApp {
    pub editor: TaffyEditor,
}

#[derive(Debug)]
pub struct TaffyEditor {
    tree: TaffyTree,
    root: taffy::NodeId,
    current_value: NodeId,
    default_style: Style,
}
impl Default for TaffyEditor {
    fn default() -> Self {
        let mut tree = TaffyTree::new();
        let mut default_style = Style::DEFAULT;
        default_style.padding = taffy::Rect {
            left: LengthPercentage::Length(10.0),
            right: LengthPercentage::Length(10.0),
            top: LengthPercentage::Length(10.0),
            bottom: LengthPercentage::Length(10.0),
        };
        default_style.margin = taffy::Rect {
            left: LengthPercentageAuto::Length(10.0),
            right: LengthPercentageAuto::Length(10.0),
            top: LengthPercentageAuto::Length(10.0),
            bottom: LengthPercentageAuto::Length(10.0),
        };
        default_style.border = taffy::Rect {
            left: LengthPercentage::Length(10.0),
            right: LengthPercentage::Length(10.0),
            top: LengthPercentage::Length(10.0),
            bottom: LengthPercentage::Length(10.0),
        };
        default_style.flex_grow = 1.0;
        default_style.box_sizing = BoxSizing::ContentBox;
        default_style.size = Size {
            width: Dimension::Auto,
            height: Dimension::Auto,
        };
        default_style.gap = taffy::Size {
            width: LengthPercentage::Length(10.0),
            height: LengthPercentage::Length(10.0),
        };
        let c0_0 = tree.new_leaf(default_style.clone()).unwrap();
        let c0_1 = tree.new_leaf(default_style.clone()).unwrap();

        let c0 = tree
            .new_with_children(
                {
                    let mut style = default_style.clone();
                    style.flex_direction = FlexDirection::Column;
                    style
                },
                &[c0_0, c0_1],
            )
            .unwrap();
        let c1 = tree.new_leaf(default_style.clone()).unwrap();
        let c2 = tree.new_leaf(default_style.clone()).unwrap();
        let root = tree
            .new_with_children(
                {
                    let mut style = default_style.clone();
                    style.size = Size {
                        width: Dimension::Length(600.0),
                        height: Dimension::Length(400.0),
                    };
                    style
                },
                &[c0, c1, c2],
            )
            .unwrap();
        Self {
            tree,
            default_style,
            root,
            current_value: root,
        }
    }
}
impl TaffyEditor {
    pub fn ui(&mut self, ctx: &egui::Context) {
        let Self {
            tree,
            root,
            current_value,
            default_style,
        } = self;
        let root = *root;
        egui::Window::new("Node Visuals")
            .default_size([600.0, 400.0])
            .show(ctx, |ui| {
                let layout = *tree.get_final_layout(root);
                ui.scope_builder(
                    UiBuilder::new()
                        .id_salt("node painter")
                        .sense(Sense::click()),
                    |ui| {
                        ui.set_min_size(egui::vec2(layout.size.width, layout.size.height));
                        let res = ui.response();
                        let offset = res.rect.min;
                        let offset = egui::vec2(offset.x, offset.y);

                        if let Some(pos) = res.hover_pos() {
                            if let Some(hover_node) = node_event_recursive(
                                tree,
                                NodeEvent::Hover(egui::vec2(pos.x, pos.y)),
                                offset,
                                root,
                            ) {
                                let hover_layout = *tree.get_final_layout(hover_node);
                                res.clone().on_hover_text(format!("{:#?}", hover_layout));
                            }
                        }
                        if let Some(pos) = res.interact_pointer_pos() {
                            if let Some(click_node) = node_event_recursive(
                                tree,
                                NodeEvent::Click(egui::vec2(pos.x, pos.y)),
                                offset,
                                root,
                            ) {
                                *current_value = click_node;
                            }
                        }
                        node_tree_paint_recursive(tree, root, ui.painter(), offset, *current_value);
                    },
                );
            });
        egui::Window::new("Node Editor")
            .default_size([600.0, 400.0])
            .scroll([true, true])
            .show(ctx, |ui| {
                SidePanel::left("node selector").show_inside(ui, |ui| {
                    node_tree_ui_recursive(ui, tree, root, current_value);
                });
                ui.indent("style editor indent", |ui| {
                    const GIT_HASH: &str = env!("VERGEN_GIT_SHA");
                    ui.label(format!("git hash: {GIT_HASH}"));

                    ui.horizontal(|ui| {
                        if ui.button("add child").clicked() {
                            let child = tree.new_leaf(default_style.clone()).unwrap();
                            tree.add_child(*current_value, child).unwrap();
                        }
                        if ui.button("delete node ").clicked() {
                            let new_current_value = tree.parent(*current_value).unwrap_or(root);
                            let _ = tree.remove(*current_value);
                            *current_value = new_current_value;
                        }
                        if ui.button("reset style").clicked() {
                            tree.set_style(*current_value, default_style.clone())
                                .unwrap();
                        }
                        let res = ui.button("print tree");
                        if res.clicked() {
                            tree.print_tree(*current_value);
                        }
                        if res.hovered() {
                            res.on_hover_text(
                                "prints node tree to the console starting from the selected node",
                            );
                        }
                    });
                    taffy_style_editor(ui, tree, *current_value)
                });
            });
        tree.compute_layout(
            root,
            Size {
                width: taffy::AvailableSpace::MinContent,
                height: taffy::AvailableSpace::MinContent,
            },
        )
        .unwrap();
    }
}

fn node_tree_ui_recursive(
    ui: &mut egui::Ui,
    tree: &mut TaffyTree,
    node_id: taffy::NodeId,
    current_selected_di: &mut taffy::NodeId,
) {
    ui.selectable_value(current_selected_di, node_id, format!("{:?}", node_id));
    if tree.child_count(node_id) != 0 {
        ui.indent(node_id, |ui| {
            for child in tree.children(node_id).unwrap_or_default() {
                node_tree_ui_recursive(ui, tree, child, current_selected_di);
            }
        });
    }
}
fn taffy_style_editor(ui: &mut egui::Ui, tree: &mut TaffyTree, node_id: taffy::NodeId) {
    let Ok(mut style) = tree.style(node_id).cloned() else {
        return;
    };
    egui::Grid::new("style editor")
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            {
                ui.label("display");

                let mut selected = match style.display {
                    taffy::Display::Block => 0,
                    taffy::Display::Flex => 1,
                    taffy::Display::Grid => 2,
                    taffy::Display::None => 3,
                };
                ComboBox::from_id_salt("display").show_index(ui, &mut selected, 4, |i| match i {
                    0 => "Block",
                    1 => "Flex",
                    2 => "Grid",
                    3 => "None",
                    _ => unreachable!(),
                });
                style.display = match selected {
                    0 => taffy::Display::Block,
                    1 => taffy::Display::Flex,
                    2 => taffy::Display::Grid,
                    3 => taffy::Display::None,
                    _ => unreachable!(),
                };
                ui.end_row();
            }
            {
                ui.label("box sizing");
                let mut selected = match style.box_sizing {
                    taffy::BoxSizing::ContentBox => 0,
                    taffy::BoxSizing::BorderBox => 1,
                };
                ComboBox::from_id_salt("box sizing").show_index(
                    ui,
                    &mut selected,
                    2,
                    |i| match i {
                        0 => "ContentBox",
                        1 => "BorderBox",
                        _ => unreachable!(),
                    },
                );
                style.box_sizing = match selected {
                    0 => taffy::BoxSizing::ContentBox,
                    1 => taffy::BoxSizing::BorderBox,
                    _ => unreachable!(),
                };
                ui.end_row();
            }
            {
                ui.label("overflow");
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        for (salt, value) in [
                            ("overflow_x", &mut style.overflow.x),
                            ("overflow_y", &mut style.overflow.y),
                        ] {
                            let mut selected = match *value {
                                taffy::Overflow::Visible => 0,
                                taffy::Overflow::Hidden => 1,
                                taffy::Overflow::Scroll => 2,
                                taffy::Overflow::Clip => 3,
                            };
                            ComboBox::from_id_salt(salt).show_index(ui, &mut selected, 4, |i| {
                                match i {
                                    0 => "Visible",
                                    1 => "Hidden",
                                    2 => "Scroll",
                                    3 => "Clip",
                                    _ => unreachable!(),
                                }
                            });
                            *value = match selected {
                                0 => taffy::Overflow::Visible,
                                1 => taffy::Overflow::Hidden,
                                2 => taffy::Overflow::Scroll,
                                3 => taffy::Overflow::Clip,
                                _ => unreachable!(),
                            };
                        }
                    });
                });
                ui.end_row();
            }
            {
                ui.label("scrollbar width");
                ui.add(DragValue::new(&mut style.scrollbar_width));
                ui.end_row();
            }
            {
                ui.label("position");
                let mut selected = match style.position {
                    taffy::Position::Relative => 0,
                    taffy::Position::Absolute => 1,
                };
                ComboBox::from_id_salt("position").show_index(ui, &mut selected, 2, |i| match i {
                    0 => "Relative",
                    1 => "Absolute",
                    _ => unreachable!(),
                });
                style.position = match selected {
                    0 => taffy::Position::Relative,
                    1 => taffy::Position::Absolute,
                    _ => unreachable!(),
                };
                ui.end_row();
            }
            {
                ui.label("inset");
                ui.push_id("inset", |ui| {
                    rect_len_percent_auto_ui(ui, &mut style.inset);
                });
                ui.end_row();
            }
            {
                ui.label("size");
                ui.push_id("size", |ui| {
                    size_dimension_ui(ui, &mut style.size);
                });
                ui.end_row();
            }
            {
                ui.label("min_size");
                ui.push_id("min_size", |ui| {
                    size_dimension_ui(ui, &mut style.min_size);
                });
                ui.end_row();
            }
            {
                ui.label("max_size");
                ui.push_id("max_size", |ui| {
                    size_dimension_ui(ui, &mut style.max_size);
                });
                ui.end_row();
            }
            {
                ui.label("aspect_ratio");
                ui.horizontal(|ui| {
                    ui.push_id("aspect_ratio", |ui| {
                        let mut enabled = style.aspect_ratio.is_some();
                        if ui.checkbox(&mut enabled, "enabled").changed() {
                            if !enabled {
                                style.aspect_ratio = None;
                            } else {
                                style.aspect_ratio = Some(1.0);
                            }
                        }
                        if let Some(ratio) = style.aspect_ratio.as_mut() {
                            ui.add(DragValue::new(ratio));
                        }
                    });
                });
                ui.end_row();
            }
            {
                ui.label("margin");
                ui.push_id("margin", |ui| {
                    rect_len_percent_auto_ui(ui, &mut style.margin)
                });
                ui.end_row();
            }
            {
                ui.label("padding");
                ui.push_id("padding", |ui| rect_len_percent_ui(ui, &mut style.padding));
                ui.end_row();
            }
            {
                ui.label("border");
                ui.push_id("border", |ui| {
                    rect_len_percent_ui(ui, &mut style.border);
                });
                ui.end_row();
            }
            {
                ui.label("align_items");
                ui.horizontal(|ui| {
                    ui.push_id("align_items", |ui| {
                        let mut enabled = style.align_items.is_some();
                        if ui.checkbox(&mut enabled, "enabled").changed() {
                            if !enabled {
                                style.align_items = None;
                            } else {
                                style.align_items = Some(AlignItems::Center);
                            }
                        }
                        if let Some(align_items) = style.align_items.as_mut() {
                            align_items_ui(ui, align_items);
                        }
                    });
                });
                ui.end_row();
            }
            {
                ui.label("align_self");
                ui.horizontal(|ui| {
                    ui.push_id("align_self", |ui| {
                        let mut enabled = style.align_self.is_some();
                        if ui.checkbox(&mut enabled, "enabled").changed() {
                            if !enabled {
                                style.align_self = None;
                            } else {
                                style.align_self = Some(AlignSelf::Center);
                            }
                        }
                        if let Some(align_self) = style.align_self.as_mut() {
                            align_items_ui(ui, align_self);
                        }
                    });
                });
                ui.end_row();
            }
            {
                ui.label("justify_items");
                ui.horizontal(|ui| {
                    ui.push_id("justify_items", |ui| {
                        let mut enabled = style.justify_items.is_some();
                        if ui.checkbox(&mut enabled, "enabled").changed() {
                            if !enabled {
                                style.justify_items = None;
                            } else {
                                style.justify_items = Some(AlignItems::Center);
                            }
                        }
                        if let Some(justify_items) = style.justify_items.as_mut() {
                            align_items_ui(ui, justify_items);
                        }
                    });
                });
                ui.end_row();
            }
            {
                ui.label("justify_self");
                ui.horizontal(|ui| {
                    ui.push_id("justify_self", |ui| {
                        let mut enabled = style.justify_self.is_some();
                        if ui.checkbox(&mut enabled, "enabled").changed() {
                            if !enabled {
                                style.justify_self = None;
                            } else {
                                style.justify_self = Some(AlignSelf::Center);
                            }
                        }
                        if let Some(justify_self) = style.justify_self.as_mut() {
                            align_items_ui(ui, justify_self);
                        }
                    });
                });
                ui.end_row();
            }
            {
                ui.label("align_content");
                ui.horizontal(|ui| {
                    ui.push_id("align_content", |ui| {
                        let mut enabled = style.align_content.is_some();
                        if ui.checkbox(&mut enabled, "enabled").changed() {
                            if !enabled {
                                style.align_content = None;
                            } else {
                                style.align_content = Some(AlignContent::Center);
                            }
                        }
                        if let Some(align_content) = style.align_content.as_mut() {
                            align_content_ui(ui, align_content);
                        }
                    });
                });
                ui.end_row();
            }
            {
                ui.label("justify_content");
                ui.horizontal(|ui| {
                    ui.push_id("justify_content", |ui| {
                        let mut enabled = style.justify_content.is_some();
                        if ui.checkbox(&mut enabled, "enabled").changed() {
                            if !enabled {
                                style.justify_content = None;
                            } else {
                                style.justify_content = Some(AlignContent::Center);
                            }
                        }
                        if let Some(justify_content) = style.justify_content.as_mut() {
                            align_content_ui(ui, justify_content);
                        }
                    });
                });
                ui.end_row();
            }
            {
                ui.label("gap");
                ui.vertical(|ui| {
                    ui.push_id("gap_width", |ui| {
                        len_percent_ui(ui, &mut style.gap.width);
                    });
                    ui.push_id("gap_height", |ui| {
                        len_percent_ui(ui, &mut style.gap.height);
                    });
                });
                ui.end_row();
            }
            {
                ui.label("text_align");
                let mut selected = match style.text_align {
                    TextAlign::Auto => 0,
                    TextAlign::LegacyLeft => 1,
                    TextAlign::LegacyRight => 2,
                    TextAlign::LegacyCenter => 3,
                };
                ComboBox::from_id_salt("text_align").show_index(
                    ui,
                    &mut selected,
                    4,
                    |i| match i {
                        0 => "Auto",
                        1 => "Left",
                        2 => "Right",
                        3 => "Center",
                        _ => unreachable!(),
                    },
                );
                style.text_align = match selected {
                    0 => TextAlign::Auto,
                    1 => TextAlign::LegacyLeft,
                    2 => TextAlign::LegacyRight,
                    3 => TextAlign::LegacyCenter,
                    _ => unreachable!(),
                };
                ui.end_row();
            }
            {
                ui.label("flex_direction");
                let mut selected = match style.flex_direction {
                    FlexDirection::Row => 0,
                    FlexDirection::Column => 1,
                    FlexDirection::RowReverse => 2,
                    FlexDirection::ColumnReverse => 3,
                };
                ComboBox::from_id_salt("flex_direction").show_index(ui, &mut selected, 4, |i| {
                    match i {
                        0 => "Row",
                        1 => "Column",
                        2 => "RowReverse",
                        3 => "ColumnReverse",
                        _ => unreachable!(),
                    }
                });
                style.flex_direction = match selected {
                    0 => FlexDirection::Row,
                    1 => FlexDirection::Column,
                    2 => FlexDirection::RowReverse,
                    3 => FlexDirection::ColumnReverse,
                    _ => unreachable!(),
                };
                ui.end_row();
            }
            {
                ui.label("flex_wrap");

                let mut selected = match style.flex_wrap {
                    taffy::FlexWrap::NoWrap => 0,
                    taffy::FlexWrap::Wrap => 1,
                    taffy::FlexWrap::WrapReverse => 2,
                };
                ComboBox::from_id_salt("flex_wrap").show_index(ui, &mut selected, 3, |i| match i {
                    0 => "NoWrap",
                    1 => "Wrap",
                    2 => "WrapReverse",
                    _ => unreachable!(),
                });
                style.flex_wrap = match selected {
                    0 => taffy::FlexWrap::NoWrap,
                    1 => taffy::FlexWrap::Wrap,
                    2 => taffy::FlexWrap::WrapReverse,
                    _ => unreachable!(),
                };
                ui.end_row();
            }
            {
                ui.label("flex_basis");
                ui.push_id("flex_basis", |ui| {
                    dimension_ui(ui, &mut style.flex_basis);
                });
                ui.end_row();
            }
            {
                ui.label("flex_grow");
                ui.add(DragValue::new(&mut style.flex_grow));
                ui.end_row();
            }
            {
                ui.label("flex_shrink");
                ui.add(DragValue::new(&mut style.flex_shrink));
                ui.end_row();
            }
        });
    tree.set_style(node_id, style).unwrap();
}
#[allow(unused)]
fn max_track_size_ui(ui: &mut egui::Ui, value: &mut taffy::MaxTrackSizingFunction) {
    let mut inner_len_percent = None;
    let mut inner = None;
    let mut selected = match value {
        MaxTrackSizingFunction::Fixed(i) => {
            inner_len_percent = Some(*i);
            0
        }
        MaxTrackSizingFunction::MinContent => 1,
        MaxTrackSizingFunction::MaxContent => 2,
        MaxTrackSizingFunction::FitContent(i) => {
            inner_len_percent = Some(*i);
            3
        }
        MaxTrackSizingFunction::Auto => 4,
        MaxTrackSizingFunction::Fraction(i) => {
            inner = Some(*i);
            5
        }
    };
    let mut inner_len_percent = inner_len_percent.unwrap_or(LengthPercentage::ZERO);
    let mut inner = inner.unwrap_or(0.0);
    ui.horizontal(|ui| {
        ComboBox::from_id_salt("max_track_size").show_index(ui, &mut selected, 6, |i| match i {
            0 => "Fixed",
            1 => "MinContent",
            2 => "MaxContent",
            3 => "FitContent",
            4 => "Auto",
            5 => "Fraction",
            _ => unreachable!(),
        });
        ui.vertical(|ui| {
            ui.add_enabled_ui(selected == 3 || selected == 0, |ui| {
                len_percent_ui(ui, &mut inner_len_percent)
            });
            ui.add_enabled_ui(selected == 5, |ui| ui.add(DragValue::new(&mut inner)));
        });
        *value = match selected {
            0 => MaxTrackSizingFunction::Fixed(inner_len_percent),
            1 => MaxTrackSizingFunction::MinContent,
            2 => MaxTrackSizingFunction::MaxContent,
            3 => MaxTrackSizingFunction::FitContent(inner_len_percent),
            4 => MaxTrackSizingFunction::Auto,
            5 => MaxTrackSizingFunction::Fraction(inner),
            _ => unreachable!(),
        };
    });
}
#[allow(unused)]
fn min_track_size_ui(ui: &mut egui::Ui, value: &mut taffy::MinTrackSizingFunction) {
    let mut inner = None;
    let mut selected = match value {
        MinTrackSizingFunction::Fixed(i) => {
            inner = Some(*i);
            0
        }
        MinTrackSizingFunction::MinContent => 1,
        MinTrackSizingFunction::MaxContent => 2,
        MinTrackSizingFunction::Auto => 3,
    };
    let mut inner = inner.unwrap_or(LengthPercentage::ZERO);
    ui.horizontal(|ui| {
        ComboBox::from_id_salt("min_track_size").show_index(ui, &mut selected, 4, |i| match i {
            0 => "Fixed",
            1 => "MinContent",
            2 => "MaxContent",
            3 => "Auto",
            _ => unreachable!(),
        });
        ui.add_enabled_ui(selected == 1, |ui| len_percent_ui(ui, &mut inner));
        *value = match selected {
            0 => MinTrackSizingFunction::Fixed(inner),
            1 => MinTrackSizingFunction::MinContent,
            2 => MinTrackSizingFunction::MaxContent,
            3 => MinTrackSizingFunction::Auto,
            _ => unreachable!(),
        };
    });
}
fn align_content_ui(ui: &mut egui::Ui, value: &mut taffy::AlignContent) {
    let mut selected = match *value {
        AlignContent::Start => 0,
        AlignContent::End => 1,
        AlignContent::FlexStart => 2,
        AlignContent::FlexEnd => 3,
        AlignContent::Center => 4,
        AlignContent::Stretch => 5,
        AlignContent::SpaceBetween => 6,
        AlignContent::SpaceEvenly => 7,
        AlignContent::SpaceAround => 8,
    };
    ComboBox::from_id_salt("align_content").show_index(ui, &mut selected, 9, |i| match i {
        0 => "Start",
        1 => "End",
        2 => "FlexStart",
        3 => "FlexEnd",
        4 => "Center",
        5 => "Stretch",
        6 => "SpaceBetween",
        7 => "SpaceEvenly",
        8 => "SpaceAround",
        _ => unreachable!(),
    });
    *value = match selected {
        0 => AlignContent::Start,
        1 => AlignContent::End,
        2 => AlignContent::FlexStart,
        3 => AlignContent::FlexEnd,
        4 => AlignContent::Center,
        5 => AlignContent::Stretch,
        6 => AlignContent::SpaceBetween,
        7 => AlignContent::SpaceEvenly,
        8 => AlignContent::SpaceAround,
        _ => unreachable!(),
    };
}
fn align_items_ui(ui: &mut egui::Ui, value: &mut taffy::AlignItems) {
    let mut selected = match *value {
        AlignItems::Start => 0,
        AlignItems::End => 1,
        AlignItems::FlexStart => 2,
        AlignItems::FlexEnd => 3,
        AlignItems::Center => 4,
        AlignItems::Baseline => 5,
        AlignItems::Stretch => 6,
    };
    ComboBox::from_id_salt("align_items").show_index(ui, &mut selected, 7, |i| match i {
        0 => "Start",
        1 => "End",
        2 => "FlexStart",
        3 => "FlexEnd",
        4 => "Center",
        5 => "Baseline",
        6 => "Stretch",
        _ => unreachable!(),
    });
    *value = match selected {
        0 => AlignItems::Start,
        1 => AlignItems::End,
        2 => AlignItems::FlexStart,
        3 => AlignItems::FlexEnd,
        4 => AlignItems::Center,
        5 => AlignItems::Baseline,
        6 => AlignItems::Stretch,
        _ => unreachable!(),
    };
}
fn rect_len_percent_ui(ui: &mut egui::Ui, value: &mut taffy::Rect<taffy::style::LengthPercentage>) {
    ui.vertical(|ui| {
        for (seed, value) in [
            ("left", &mut value.left),
            ("right", &mut value.right),
            ("top", &mut value.top),
            ("bottom", &mut value.bottom),
        ] {
            ui.push_id(seed, |ui| {
                ui.horizontal(|ui| {
                    ui.label(seed);
                    len_percent_ui(ui, value);
                });
            });
        }
    });
}
fn len_percent_ui(ui: &mut egui::Ui, value: &mut LengthPercentage) {
    let mut inner;
    let mut selected = match value {
        LengthPercentage::Length(i) => {
            inner = *i;
            0
        }
        LengthPercentage::Percent(i) => {
            inner = *i;
            1
        }
    };
    ui.horizontal(|ui| {
        ComboBox::from_id_salt("length_percent").show_index(ui, &mut selected, 2, |i| match i {
            0 => "Length",
            1 => "Percent",
            _ => unreachable!(),
        });

        ui.add(DragValue::new(&mut inner));
        if selected == 0 {
            *value = LengthPercentage::Length(inner);
        } else {
            *value = LengthPercentage::Percent(inner);
        }
    });
}
fn size_dimension_ui(ui: &mut egui::Ui, value: &mut taffy::Size<taffy::Dimension>) {
    ui.vertical(|ui| {
        ui.push_id("width", |ui| dimension_ui(ui, &mut value.width));
        ui.push_id("height", |ui| dimension_ui(ui, &mut value.height));
    });
}
fn dimension_ui(ui: &mut egui::Ui, value: &mut taffy::Dimension) {
    let mut inner = 0.0;
    let mut selected = match value {
        Dimension::Length(i) => {
            inner = *i;
            0
        }
        Dimension::Percent(i) => {
            inner = *i;
            1
        }
        Dimension::Auto => 2,
    };
    ui.horizontal(|ui| {
        ComboBox::from_id_salt("dimension").show_index(ui, &mut selected, 3, |i| match i {
            0 => "Length",
            1 => "Percent",
            2 => "Auto",
            _ => unreachable!(),
        });

        ui.add_enabled(selected != 2, DragValue::new(&mut inner));

        *value = match selected {
            0 => Dimension::Length(inner),
            1 => Dimension::Percent(inner),
            2 => Dimension::Auto,
            _ => unreachable!(),
        };
    });
}
fn rect_len_percent_auto_ui(
    ui: &mut egui::Ui,
    value: &mut taffy::Rect<taffy::style::LengthPercentageAuto>,
) {
    ui.vertical(|ui| {
        for (seed, value) in [
            ("left", &mut value.left),
            ("right", &mut value.right),
            ("top", &mut value.top),
            ("bottom", &mut value.bottom),
        ] {
            ui.horizontal(|ui| {
                ui.push_id(seed, |ui| {
                    ui.label(seed);
                    len_percent_auto_ui(ui, value);
                });
            });
        }
    });
}
fn len_percent_auto_ui(ui: &mut egui::Ui, value: &mut taffy::style::LengthPercentageAuto) {
    let mut inner = 0.0;
    let mut selected = match value {
        taffy::LengthPercentageAuto::Length(i) => {
            inner = *i;
            0
        }
        taffy::LengthPercentageAuto::Percent(i) => {
            inner = *i;
            1
        }
        taffy::LengthPercentageAuto::Auto => 2,
    };
    ui.horizontal(|ui| {
        ComboBox::from_id_salt("length_percent_auto").show_index(
            ui,
            &mut selected,
            3,
            |i| match i {
                0 => "Length",
                1 => "Percent",
                2 => "Auto",
                _ => unreachable!(),
            },
        );
        ui.add_enabled(selected != 2, DragValue::new(&mut inner));
        match selected {
            0 => {
                *value = taffy::LengthPercentageAuto::Length(inner);
            }
            1 => {
                *value = taffy::LengthPercentageAuto::Percent(inner);
            }
            2 => *value = taffy::LengthPercentageAuto::Auto,
            _ => unreachable!(),
        }
    });
}

#[derive(Debug, Copy, Clone)]
enum NodeEvent {
    Hover(Vec2),
    Click(Vec2),
}
/// This takes a taffy node, checks if the event is consumed by any of its children.
/// If it is not consumed, it will check if the event is consumed by the node itself.
/// If it is consumed, it will return the node id of self.
/// If it is consumed by one of the children, then it will return the returned node id.
/// If it is not consumed by any of the children or itself, it will return None.
fn node_event_recursive(
    tree: &mut TaffyTree,
    ev: NodeEvent,
    offset: Vec2,
    node_id: taffy::NodeId,
) -> Option<NodeId> {
    let layout = *tree.get_final_layout(node_id);

    let new_offset = offset + egui::vec2(layout.location.x, layout.location.y);

    let mut children = tree.children(node_id).unwrap_or_default();
    children.sort_unstable_by_key(|i| tree.get_final_layout(*i).order);
    for child in children {
        if let Some(new_node_id) = node_event_recursive(tree, ev, new_offset, child) {
            return Some(new_node_id);
        }
    }

    let node_rect = egui::Rect::from_min_size(
        [layout.location.x, layout.location.y].into(),
        [layout.size.width, layout.size.height].into(),
    )
    .translate(offset);
    match ev {
        NodeEvent::Hover(vec2) => {
            if node_rect.contains(egui::pos2(vec2.x, vec2.y)) {
                return Some(node_id);
            }
        }
        NodeEvent::Click(vec2) => {
            if node_rect.contains(egui::pos2(vec2.x, vec2.y)) {
                return Some(node_id);
            }
        }
    }
    None
}
fn node_tree_paint_recursive(
    tree: &TaffyTree,
    node_id: taffy::NodeId,
    painter: &Painter,
    offset: Vec2,
    focused_node: NodeId,
) {
    let layout = *tree.get_final_layout(node_id);
    let node_rect = egui::Rect::from_min_size(
        [layout.location.x, layout.location.y].into(),
        [layout.size.width, layout.size.height].into(),
    );
    let margin_rect = node_rect.translate(offset);
    /// Takes a rect, shrinks it by cutting the respective side with values from the cuts and gives us the sub rect
    fn get_sub_rect(rect: egui::Rect, cuts: taffy::Rect<f32>) -> egui::Rect {
        egui::Rect::from_min_max(
            egui::pos2(rect.min.x + cuts.left, rect.min.y + cuts.top),
            egui::pos2(rect.max.x - cuts.right, rect.max.y - cuts.bottom),
        )
    }
    painter.rect_filled(
        margin_rect,
        0.0,
        Color32::from_hex("#ff7046").unwrap_or_default(),
    );
    if focused_node == node_id {
        painter.add(egui::Shape::dashed_line(
            &[
                margin_rect.left_top(),
                margin_rect.right_top(),
                margin_rect.right_bottom(),
                margin_rect.left_bottom(),
                margin_rect.left_top(),
            ],
            Stroke::new(5.0, Color32::DEBUG_COLOR),
            10.0,
            10.0,
        ));
        // painter.rect_stroke(margin_rect, 3.0, Stroke::new(5.0, Color32::RED));
    }
    let border_rect = get_sub_rect(margin_rect, layout.margin);
    painter.rect_filled(
        border_rect,
        0.0,
        Color32::from_hex("#00a6c3").unwrap_or_default(),
    );
    let padding_rect = get_sub_rect(border_rect, layout.border);
    painter.rect_filled(
        padding_rect,
        0.0,
        Color32::from_hex("#fac357").unwrap_or_default(),
    );
    let content_rect = get_sub_rect(padding_rect, layout.padding);
    painter.rect_filled(
        content_rect,
        0.0,
        Color32::from_hex("#00c4a8").unwrap_or_default(),
    );
    let new_offset = offset + egui::vec2(layout.location.x, layout.location.y);
    if tree.child_count(node_id) != 0 {
        let mut children = tree.children(node_id).unwrap_or_default();
        children.sort_unstable_by_key(|i| tree.get_final_layout(*i).order);
        for child in children {
            node_tree_paint_recursive(tree, child, painter, new_offset, focused_node);
        }
    }
}
impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.editor.ui(ctx);
    }
}

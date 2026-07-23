use crate::draw::DrawProperties;

pub mod block;
pub mod commons;
pub mod draw;
pub mod flex;
pub mod grid;
pub mod layout;

#[derive(Debug, Clone)]
pub struct RenderRect {
    pub node_id: taffy::NodeId,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub depth: u32,    // handy for color-by-depth or indentation in either renderer
    pub label: String, // e.g. "Flex" / "Other(t)" — whatever you want shown on hover
    pub style_str: String,

    pub draw: Option<DrawProperties>,
}

pub fn collect_rects(
    tree: &taffy::TaffyTree<layout::LeafContext>,
    root: taffy::NodeId,
) -> eyre::Result<Vec<RenderRect>> {
    let mut out = Vec::new();
    collect_rects_inner(tree, root, 0.0, 0.0, 0, &mut out)?;
    Ok(out)
}

fn collect_rects_inner(
    tree: &taffy::TaffyTree<layout::LeafContext>,
    node: taffy::NodeId,
    parent_x: f32,
    parent_y: f32,
    depth: u32,
    out: &mut Vec<RenderRect>,
) -> eyre::Result<()> {
    let layout = tree.layout(node)?;
    let x = parent_x + layout.location.x;
    let y = parent_y + layout.location.y;

    let defctx = layout::LeafContext {
        ..Default::default()
    };
    let ctx = tree.get_node_context(node).unwrap_or(&defctx);

    out.push(RenderRect {
        node_id: node,
        x,
        y,
        width: layout.size.width,
        height: layout.size.height,
        depth,
        style_str: ctx.debug_str.clone(),
        label: format!("{}", ctx.id.clone().unwrap_or(u64::from(node).to_string())),
        draw: ctx.draw.clone(),
    });

    for child in tree.children(node)? {
        collect_rects_inner(tree, child, x, y, depth + 1, out)?;
    }

    Ok(())
}

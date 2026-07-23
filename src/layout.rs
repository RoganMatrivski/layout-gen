use std::{convert::TryFrom, fmt::Debug};

use eyre::Context;
use roxmltree::{Document, Node};
use strum::EnumString;
use taffy::TaffyTree;

use crate::{
    block::BlockProperties,
    commons::{FromXmlAttrs, LeafProperties},
    draw::DrawProperties,
    flex::FlexProperties,
    grid::GridProperties,
};

#[derive(EnumString, Debug, Clone)]
pub enum Leaf {
    Flex(LeafStruct<FlexProperties>),
    Block(LeafStruct<BlockProperties>),
    Grid(LeafStruct<GridProperties>),
    // Draw(LeafStruct<DrawProperties>),
    Other(String),
}

#[derive(Debug, Clone, Default)]
pub struct LeafContext {
    pub id: Option<String>,
    // TODO: Find something better than this
    pub debug_str: String,
    pub draw: Option<DrawProperties>,
}

impl Leaf {
    fn build_leaf<P: LeafProperties + Debug>(
        tree: &mut TaffyTree<LeafContext>,
        leaf: LeafStruct<P>,
        kind: &str,
    ) -> eyre::Result<taffy::NodeId> {
        let style = leaf.props.to_taffy_style();
        let child_leaves = leaf
            .children
            .into_iter()
            .map(|x| x.build_taffy_tree(tree))
            .collect::<Result<Vec<_>, eyre::Error>>()?;

        let this_leaf = tree
            .new_with_children(style, &child_leaves)
            .wrap_err_with(|| format!("Failed to create {kind} leaf"))?;

        tree.set_node_context(
            this_leaf,
            Some(LeafContext {
                id: leaf.props.id(),
                debug_str: format!("{:#?}", leaf.props),
                draw: leaf.draw, // new
            }),
        )
        .wrap_err("Failed to set node context")?;

        Ok(this_leaf)
    }

    pub fn build_taffy_tree(
        self,
        tree: &mut TaffyTree<LeafContext>,
    ) -> eyre::Result<taffy::NodeId> {
        match self {
            Leaf::Flex(leaf) => Self::build_leaf(tree, leaf, "flex"),
            Leaf::Block(leaf) => Self::build_leaf(tree, leaf, "block"),
            Leaf::Grid(leaf) => Self::build_leaf(tree, leaf, "grid"),
            Leaf::Other(tag) => {
                tracing::warn!("Unknown tag type {tag}. Replacing with default tag");
                tree.new_leaf(taffy::Style::default())
                    .wrap_err("Failed to create unknown leaf")
            }
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct LeafStruct<T> {
    props: T,
    children: Vec<Leaf>,
    draw: Option<DrawProperties>, // new — not a taffy child
}

impl<'doc, 'input, P> TryFrom<Node<'doc, 'input>> for LeafStruct<P>
where
    P: FromXmlAttrs + Default + Send,
{
    type Error = eyre::Error;

    fn try_from(node: Node<'doc, 'input>) -> Result<Self, Self::Error> {
        let defaults = P::default();
        let props = P::from_node(node, &defaults).map_err(|e| eyre::eyre!("{e}"))?;

        let mut children = Vec::new();
        let mut draw = None;

        for child in node.children().filter(|x| x.is_element()) {
            if child.tag_name().name() == "draw" {
                let d_defaults = DrawProperties::default();
                if draw.is_some() {
                    eyre::bail!(
                        "Node '{}' has more than one <draw> child; only one is allowed",
                        node.attribute("id").unwrap_or("<unnamed>")
                    );
                }
                draw = Some(DrawProperties::from_node(child, &d_defaults)?);
            } else {
                children.push(parse_node(child)?);
            }
        }

        Ok(Self {
            children,
            props,
            draw,
        })
    }
}

pub fn parse_node(xml_node: Node) -> eyre::Result<Leaf> {
    let leaf_tag = xml_node.tag_name().name();
    let leaf = match leaf_tag {
        "flex" => Leaf::Flex(LeafStruct::try_from(xml_node)?),
        "block" => Leaf::Block(LeafStruct::try_from(xml_node)?),
        "grid" => Leaf::Grid(LeafStruct::try_from(xml_node)?),
        other => Leaf::Other(other.to_string()),
    };

    Ok(leaf)
}

#[derive(Debug, Clone)]
pub struct Layout {
    version: String,
    children: Vec<Leaf>,
}

pub fn parse_layout(xmlstr: &str) -> eyre::Result<Layout> {
    let doc = Document::parse(xmlstr)?;
    let root = doc.root_element();

    Ok(Layout {
        version: root
            .attribute("version")
            .map(String::from)
            .unwrap_or("0.1.0".into()),
        children: root
            .children()
            .filter(|x| x.is_element())
            .map(parse_node)
            .collect::<Result<Vec<_>, eyre::Error>>()?,
    })
}

impl Layout {
    pub fn build_taffy_tree(
        self,
        tree: &mut TaffyTree<LeafContext>,
    ) -> eyre::Result<taffy::NodeId> {
        let child_leafs = self
            .children
            .into_iter()
            .map(|x| x.build_taffy_tree(tree))
            .collect::<Result<Vec<_>, eyre::Error>>()?;

        let root_style = taffy::Style {
            display: taffy::Display::Flex,
            flex_direction: taffy::FlexDirection::Column,
            size: taffy::Size {
                width: taffy::Dimension::percent(1.0),
                height: taffy::Dimension::percent(1.0),
            },
            ..Default::default()
        };

        tree.new_with_children(root_style, &child_leafs)
            .wrap_err("Failed to create root leaf")
    }
}

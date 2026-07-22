use std::convert::TryFrom;

use eyre::Context;
use roxmltree::{Document, Node};
use strum::EnumString;
use taffy::TaffyTree;

use crate::{
    block::BlockProperties, commons::FromXmlAttrs, flex::FlexProperties, grid::GridProperties,
};

#[derive(EnumString, Debug, Clone)]
pub enum Leaf {
    Flex(LeafStruct<FlexProperties>),
    Block(LeafStruct<BlockProperties>),
    Grid(LeafStruct<GridProperties>),
    Other(String),
}

impl Leaf {
    pub fn build_taffy_tree(self, tree: &mut TaffyTree) -> eyre::Result<taffy::NodeId> {
        match self {
            Leaf::Flex(flex_leaf) => {
                let leaf_style = flex_leaf.props.to_taffy_style();

                let child_leaves = flex_leaf
                    .children
                    .into_iter()
                    .map(|x| x.build_taffy_tree(tree))
                    .collect::<Result<Vec<_>, eyre::Error>>()?;

                tree.new_with_children(leaf_style, &child_leaves)
                    .wrap_err("Failed to create flex leaf")
            }
            Leaf::Block(block_leaf) => {
                let leaf_style = block_leaf.props.to_taffy_style();

                let child_leaves = block_leaf
                    .children
                    .into_iter()
                    .map(|x| x.build_taffy_tree(tree))
                    .collect::<Result<Vec<_>, eyre::Error>>()?;

                tree.new_with_children(leaf_style, &child_leaves)
                    .wrap_err("Failed to create block leaf")
            }
            Leaf::Grid(grid_leaf) => {
                let leaf_style = grid_leaf.props.to_taffy_style();

                let child_leaves = grid_leaf
                    .children
                    .into_iter()
                    .map(|x| x.build_taffy_tree(tree))
                    .collect::<Result<Vec<_>, eyre::Error>>()?;

                tree.new_with_children(leaf_style, &child_leaves)
                    .wrap_err("Failed to create grid leaf")
            }
            Leaf::Other(tag) => {
                tracing::warn!("Unknown tag type {tag}. Replacing with default tag");
                tree.new_leaf(taffy::Style::default())
                    .wrap_err("Failed to create unknown leaf")
            }
            // This arm is for catch-all when any leaf added
            #[allow(unused)]
            leaf => unimplemented!(),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct LeafStruct<T> {
    props: T,
    children: Vec<Leaf>,
}

impl<'doc, 'input, P> TryFrom<Node<'doc, 'input>> for LeafStruct<P>
where
    P: FromXmlAttrs + Default + Send,
{
    type Error = eyre::Error;

    fn try_from(node: Node<'doc, 'input>) -> Result<Self, Self::Error> {
        let defaults = P::default();
        let props = P::from_node(node, &defaults).map_err(|e| eyre::eyre!("{e}"))?;

        let children = node
            .children()
            .filter(|x| x.is_element())
            .map(parse_node)
            .collect::<Result<Vec<Leaf>, Self::Error>>()?;

        Ok(Self { children, props })
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
    pub fn build_taffy_tree(self, tree: &mut TaffyTree) -> eyre::Result<taffy::NodeId> {
        let child_leafs = self
            .children
            .into_iter()
            .map(|x| x.build_taffy_tree(tree))
            .collect::<Result<Vec<_>, eyre::Error>>()?;

        let root_style = taffy::Style {
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

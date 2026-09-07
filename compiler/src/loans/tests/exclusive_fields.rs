use super::authority::inspect;
use crate::borrow::Source;
use crate::loans::{Graph, Step};

#[test]
pub(crate) fn primary_access_excludes_named_descendants_but_retains_ancestors() {
    inspect(
        "v:={->1;->inner:={->2;->n:=3}};p:&!v.inner.n;a<int32>:v;b<int32>:v.inner;w:*p",
        |graph, _| {
            let borrowed = &graph.values[graph.loans[0].value].origins[0].source;
            let Source::Local { id, fields } = borrowed else {
                panic!("ordinary field owner");
            };
            assert_eq!(fields.len(), 2);
            let inner = Source::Local {
                id: *id,
                fields: fields[..1].to_vec(),
            };
            let owner = Source::Local {
                id: *id,
                fields: Vec::new(),
            };
            let region = graph
                .nodes
                .iter()
                .filter_map(|node| node.access.as_ref())
                .flat_map(|access| &access.regions)
                .find(|region| region.component == [Step::Slot(0)] && region.source == inner)
                .unwrap();
            assert!(Graph::physical_overlap(&region.source, borrowed));
            assert!(!Graph::access_overlap(region, borrowed));
            assert!(Graph::access_overlap(region, &inner));
            assert!(Graph::access_overlap(region, &owner));
        },
    );
}

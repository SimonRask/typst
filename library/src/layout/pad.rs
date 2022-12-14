use crate::prelude::*;

/// Pad content at the sides.
#[func]
#[capable(Layout)]
#[derive(Debug, Hash)]
pub struct PadNode {
    /// The amount of padding.
    pub padding: Sides<Rel<Length>>,
    /// The content whose sides to pad.
    pub body: Content,
}

#[node]
impl PadNode {
    fn construct(_: &Vm, args: &mut Args) -> SourceResult<Content> {
        let all = args.named("rest")?.or(args.find()?);
        let x = args.named("x")?;
        let y = args.named("y")?;
        let left = args.named("left")?.or(x).or(all).unwrap_or_default();
        let top = args.named("top")?.or(y).or(all).unwrap_or_default();
        let right = args.named("right")?.or(x).or(all).unwrap_or_default();
        let bottom = args.named("bottom")?.or(y).or(all).unwrap_or_default();
        let body = args.expect::<Content>("body")?;
        let padding = Sides::new(left, top, right, bottom);
        Ok(Self { padding, body }.pack())
    }
}

impl Layout for PadNode {
    fn layout(
        &self,
        vt: &mut Vt,
        styles: StyleChain,
        regions: Regions,
    ) -> SourceResult<Fragment> {
        let mut backlog = vec![];

        // Layout child into padded regions.
        let padding = self.padding.resolve(styles);
        let pod = regions.map(&mut backlog, |size| shrink(size, padding));
        let mut fragment = self.body.layout(vt, styles, pod)?;

        for frame in &mut fragment {
            // Apply the padding inversely such that the grown size padded
            // yields the frame's size.
            let padded = grow(frame.size(), padding);
            let padding = padding.relative_to(padded);
            let offset = Point::new(padding.left, padding.top);

            // Grow the frame and translate everything in the frame inwards.
            frame.set_size(padded);
            frame.translate(offset);
        }

        Ok(fragment)
    }
}

/// Shrink a size by padding relative to the size itself.
fn shrink(size: Size, padding: Sides<Rel<Abs>>) -> Size {
    size - padding.relative_to(size).sum_by_axis()
}

/// Grow a size by padding relative to the grown size.
/// This is the inverse operation to `shrink()`.
///
/// For the horizontal axis the derivation looks as follows.
/// (Vertical axis is analogous.)
///
/// Let w be the grown target width,
///     s be the given width,
///     l be the left padding,
///     r be the right padding,
///     p = l + r.
///
/// We want that: w - l.resolve(w) - r.resolve(w) = s
///
/// Thus: w - l.resolve(w) - r.resolve(w) = s
///   <=> w - p.resolve(w) = s
///   <=> w - p.rel * w - p.abs = s
///   <=> (1 - p.rel) * w = s + p.abs
///   <=> w = (s + p.abs) / (1 - p.rel)
fn grow(size: Size, padding: Sides<Rel<Abs>>) -> Size {
    size.zip(padding.sum_by_axis())
        .map(|(s, p)| (s + p.abs).safe_div(1.0 - p.rel.get()))
}

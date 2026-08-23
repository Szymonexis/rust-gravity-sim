use super::ColorRGBA;

type ColorHSLA = [f32; 4];

pub fn mix(from: ColorRGBA, to: ColorRGBA, delta: f32) -> ColorRGBA {
    let delta = delta.clamp(0.0, 1.0);

    let [from_hue, from_saturation, from_lightness, from_alpha] = to_hsla(from);
    let [to_hue, to_saturation, to_lightness, to_alpha] = to_hsla(to);

    let from_hue = if from_saturation == 0.0 {
        to_hue
    } else {
        from_hue
    };
    let to_hue = if to_saturation == 0.0 {
        from_hue
    } else {
        to_hue
    };

    let mut hue_step = to_hue - from_hue;
    if hue_step > 0.5 {
        hue_step -= 1.0;
    } else if hue_step < -0.5 {
        hue_step += 1.0;
    }

    to_rgba([
        (from_hue + hue_step * delta).rem_euclid(1.0),
        lerp(from_saturation, to_saturation, delta),
        lerp(from_lightness, to_lightness, delta),
        lerp(from_alpha, to_alpha, delta),
    ])
}

fn lerp(from: f32, to: f32, delta: f32) -> f32 {
    from + (to - from) * delta
}

fn to_hsla(rgba: ColorRGBA) -> ColorHSLA {
    let [r, g, b, a] = rgba;
    let cmax = [r, g, b].into_iter().reduce(f32::max).unwrap();
    let cmin = [r, g, b].into_iter().reduce(f32::min).unwrap();
    let chroma = cmax - cmin;

    let lightness = (cmax + cmin) / 2.0;

    if chroma == 0.0 {
        return [0.0, 0.0, lightness, a];
    }

    let saturation = chroma / (1.0 - (2.0 * lightness - 1.0).abs());

    let mut hue = if cmax == r {
        (g - b) / chroma
    } else if cmax == g {
        (b - r) / chroma + 2.0
    } else {
        (r - g) / chroma + 4.0
    };
    if hue < 0.0 {
        hue += 6.0;
    }

    [hue / 6.0, saturation, lightness, a]
}

fn to_rgba(hsla: ColorHSLA) -> ColorRGBA {
    let [hue, saturation, lightness, a] = hsla;
    let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let sector = hue.rem_euclid(1.0) * 6.0;
    let x = chroma * (1.0 - (sector % 2.0 - 1.0).abs());

    let [r, g, b] = match sector as u32 {
        0 => [chroma, x, 0.0],
        1 => [x, chroma, 0.0],
        2 => [0.0, chroma, x],
        3 => [0.0, x, chroma],
        4 => [x, 0.0, chroma],
        _ => [chroma, 0.0, x],
    };

    let m = lightness - chroma / 2.0;
    [r + m, g + m, b + m, a]
}

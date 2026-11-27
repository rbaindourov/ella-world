shader_type spatial;
render_mode unshaded, cull_disabled, blend_mix, depth_draw_opaque;

uniform vec4 primary_color : source_color = vec4(0.22, 0.74, 0.97, 1.0);
uniform vec4 secondary_color : source_color = vec4(0.39, 0.40, 0.95, 1.0);
uniform float swirl_speed : hint_range(0.2, 6.0) = 2.1;

void fragment() {
	vec2 uv = UV * 2.0 - 1.0;
	float r = length(uv);
	if (r > 1.0) {
		discard;
	}
	float theta = atan(uv.y, uv.x);
	float spiral = sin(theta * 5.0 - r * 11.0 + TIME * swirl_speed * 2.4);
	float ripple = cos(r * 16.0 - TIME * swirl_speed * 3.1);
	float energy = clamp(0.55 + 0.28 * spiral + 0.22 * ripple, 0.0, 1.0);
	float rim = smoothstep(0.62, 0.98, r);
	vec3 col = mix(secondary_color.rgb, primary_color.rgb, energy);
	col = mix(col, vec3(0.92, 0.97, 1.0), rim * 0.65 + (1.0 - smoothstep(0.0, 0.26, r)) * 0.55);
	ALBEDO = col * 1.35;
	ALPHA = clamp(0.86 + rim * 0.14, 0.0, 1.0);
}

// Shamelessly ripped from:
// https://stackoverflow.com/questions/47376499/creating-a-gradient-color-in-fragment-shader

uniform vec2 res;
uniform float t, cx, cy;

void main() {

  vec2 st = gl_FragCoord.xy/res.xy;

  vec3 color1 = vec3(1.9-t,0.55+t,1.0-t);
  vec3 color2 = vec3(0.226-t,0.000,0.615+t);

  float mixValue = distance(st,vec2(cx,cy));
  vec3 color = mix(color1,color2,mixValue);

  gl_FragColor = vec4(color,mixValue);
}
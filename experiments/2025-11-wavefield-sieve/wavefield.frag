precision highp float;
uniform float u_step;


float indexAtX(float x){
// map uv.x [0,1] to indices 1..N
float n = floor(x * u_N) + 1.0;
return n;
}


void main(){
vec2 uv = vUv;
float n = indexAtX(uv.x);


// sample crossed state from 1D texture. texture coord: x = n/u_N
float tx = (n - 0.5) / u_N;
float crossed = texture2D(u_crossed, vec2(tx, 0.5)).r;


// compute wavefield height from spawned primes
float height = 0.0;
float crossAlpha = 0.0;
for(int i=0;i<128;i++){
if(i >= u_primeCount) break;
float p = u_primes[i];
float spawn = u_spawns[i];
float dt = u_time - spawn;
if(dt < 0.0) continue;
float k = 6.28318530718 / p;
float v = 6.0; // speed
float phase = k * n - v * dt;
float frontPos = v * dt;
float pulse = exp(-0.006 * pow((n - frontPos), 2.0));
height += pulse * sin(phase) / sqrt(max(1.0,p));
// subtle hit detection for multiples
float modN = mod(n, p);
if(modN < 0.6 || abs(modN - p) < 0.6){
float crest = abs(sin(phase));
float hit = smoothstep(0.995, 0.9999, crest);
crossAlpha += hit * pulse;
}
}


// base color
vec3 base = vec3(0.97 - 0.2*uv.y, 0.95, 1.0);
base *= 1.0 + 0.02 * height;


// if crossed (persistent), darken
base = mix(base, vec3(0.12,0.12,0.12), clamp(crossed,0.0,1.0));
// temporary hit flash (visual only)
base = mix(base, vec3(1.0,0.9,0.3), clamp(crossAlpha,0.0,1.0));


// highlight primes that haven't been crossed: sample crossed and also check small list of primes
// (we rely on CPU to set crossed flags so primes remain lit)


gl_FragColor = vec4(base, 1.0);
Slightly brighten the wave field for clearer timeline effect: vec3 base = vec3(0.98 - 0.15*uv.y, 0.96, 1.0);
}

module test;

import prelude;

struct Constants {
    float elapsed_secs;
    uint2 image_size;
}

struct PresentInfo {
  ConstantBuffer<Constants> constants;
  RWTexture2D<float4> image;

  void present(uint2 px) {
    let uv = px / float2(constants.image_size);

    let col = 0.5 + 0.5 * cos(constants.elapsed_secs + uv.xyx +
                              float3(0.0, 2.0, 4.0));

    image[px] = float4(col, 1.0);
  }
}

// Here is implementation of math functions/constanst that repeating in code
#define PI 3.1415926535897932384626433832795

static float EARTH_RADIUS = 6371e3;
static float ATMOSPHERE_RADIUS = 6471e3;
static float MAX_SCATTER_DISTANCE = (ATMOSPHERE_RADIUS - EARTH_RADIUS) * 3.0;

static float3 BETA_R = float3(5.8e-6f, 13.5e-6f, 33.1e-6f);
static float3 BETA_M = float3(21e-6f);

static float H_R = 8e3;
static float H_M = 1.2e3;
static float ABSORPTION_SCALE_HEIGHT = 30e3;
static float3 BETA_ABSORPTION = float3(2.04e-5, 4.97e-5, 1.95e-6);
static float FALL_OFF_ABSORPTION = 4e3;

static float3 DEFAULT_COLOR = float3(0.01);
static float BASE_INTENSITY = 5.0;

static int INTEGRAL_STEP_COUNT = 32;
static int LIGHT_INTEGRAL_STEP_COUNT = 8;

struct AtmosphereRendering {
  ConstantBuffer<Constants> constants;
  RWTexture2D<float4> image;

  void render(uint2 px) {
    var uv = float2(px.x, constants.image_size.y - px.y) /
                 float2(constants.image_size) -
             float2(0.5);
    uv.x *= float(constants.image_size.x) / float(constants.image_size.y);

    let offset_time = 3.7;
    let offset = (1.0 - cos(offset_time * 0.1)) * ATMOSPHERE_RADIUS;
    let ro = float3(0.0, EARTH_RADIUS + 1.0, 0.0);

    let rd = normalize(float3(uv.xy, 1.0) + float3(0.2, 0.3, 0.0));

    let light_dir =
        normalize(float3(0.0, (sin(constants.elapsed_secs * 0.5) - 1.0) * 0.5,
                         sin(constants.elapsed_secs * 0.5)));

    // Realistic day-night cycle (24 second day for demo)
    float day_duration = 12.0; // seconds for full day cycle
    float time_of_day =
        fmod(constants.elapsed_secs, day_duration) / day_duration;

    // Sun moves from -90° (midnight) to +90° (noon) and back
    float sun_elevation = 180.0 * time_of_day - 90.0; // -90 to +90 degrees
    float sun_azimuth = 180.0;                        // Fixed azimuth (south)

    let sun_dir =
        sun_angles_to_direction(float3(sun_azimuth, sun_elevation, 0.0));
    // let sun_dir = sun_angles_to_direction(float3(240.0, 0.1, 0.0));

    var col = compute_incident_light(sun_dir, ro, rd);
    col = 1.0 - exp(-col);
    image[px] = float4(col, 1.0);
  }

  float3 compute_incident_light(float3 sun_dir, float3 o, float3 d) {

    // Trace ray from viewer position to atmosphere
    let intersection =
        ray_sphere_interesect(o, d, float3(0.0), ATMOSPHERE_RADIUS);

    if (intersection.x > intersection.y)
      return DEFAULT_COLOR;

    var hit_distance = max(intersection.x, intersection.y);
    if (hit_distance < 0.0)
      return DEFAULT_COLOR;

    hit_distance = min(hit_distance, MAX_SCATTER_DISTANCE);

    // Calculate point where ray hit a sphere
    let hit_pos = d * hit_distance;
    let ray_step = hit_pos / float(INTEGRAL_STEP_COUNT);
    let step_height = length(ray_step);

    var sum_r = float3(0.0);
    var sum_m = float3(0.0);
    var optical_depth = float3(0.0);

    for (int i = 0; i < INTEGRAL_STEP_COUNT; ++i) {
      // Calculate current ray step from VIEWER to SKY
      let sample_position = o + ray_step * float(i);
      let height = length(sample_position) - EARTH_RADIUS;

      // compute optical depth for light
      let density = calculate_density(height, step_height);
      optical_depth += density;

      // Trace ray from current integral position to sun direction
      let sun_intersection = ray_sphere_interesect(
          sample_position, -sun_dir, float3(0.0), ATMOSPHERE_RADIUS);
      let sun_hit_distance = max(sun_intersection.x, sun_intersection.y);

      // Calculate point where ray hit a sphere
      let sun_hit_pos = -sun_dir * sun_hit_distance;
      let sun_ray_step = sun_hit_pos / float(LIGHT_INTEGRAL_STEP_COUNT);
      let sun_step_height = length(sun_ray_step);

      var sun_optical_depth = float3(0.0);

      for (int j = 0; j < LIGHT_INTEGRAL_STEP_COUNT; ++j) {
        // Calculate position from INTEGRAL POSITION to SUN
        let sun_integral_pos = sample_position + sun_ray_step * float(j);
        let sun_integral_height = length(sun_integral_pos) - EARTH_RADIUS;
        sun_optical_depth +=
            calculate_density(sun_integral_height, sun_step_height);
      }

      let attenuation =
          exp(-(BETA_R * (optical_depth.x + sun_optical_depth.x) +
                BETA_M * 1.1 * (optical_depth.y + sun_optical_depth.y) +
                BETA_ABSORPTION * (optical_depth.z + sun_optical_depth.z)));
      sum_r += attenuation * density.x;
      sum_m += attenuation * density.y;
    }

    // mu in the paper which is the cosine of the angle between the sun
    // direction and the ray direction
    let mu = dot(d, -sun_dir);

    let sun_intensity = 2.0;
    let rayleigh = BASE_INTENSITY * phase_rayleigh(mu) * BETA_R * sum_r;
    let mie = sun_intensity * phase_mie(mu, 0.76) * BETA_M * sum_m;
    let sun = sun_intensity * phase_mie(mu, 0.999) * BETA_M * sum_m;

    return rayleigh + mie + sun;
  }

  static float3 calculate_density(float height, float delta_height) {
    let scale_height = float2(H_R, H_M);

    // Calculate density for Rayleight and Mie both
    var density = float3(exp(-height / scale_height), 0.0);

    // Calculate adsorption (Ozone)
    let denom = (ABSORPTION_SCALE_HEIGHT - height) / FALL_OFF_ABSORPTION;
    density.z = (1.0 / (denom * denom + 1.0)) * density.x;

    density *= delta_height;

    return density;
  }

  static float phase_rayleigh(float cos_angle) {
    float cos_angle_square = cos_angle * cos_angle;
    return ((3.0 * PI) / 16.0) * (1.0 + cos_angle_square);
  }

  static float phase_h_g(float c, float g) {
    return (1.0 - g * g) / (4.0 * PI * pow(1.0 + g * g - 2.0 * g * c, 1.5));
  }

  static float phase_draine(float c, float alpha, float g) {
    return 1.0 / (4.0 * PI) * (1.0 - g * g) /
           pow(1.0 + g * g - 2.0 * g * c, 3.0 / 2.0) * (1.0 + alpha * c * c) /
           (1.0 + alpha * (1.0 + 2.0 * g * g) / 3.0);
  }

  static float phase_mie(float c, float g) {
    float d = 5.0;
    float gHG = exp(-0.0990567 / (d - 1.67154));
    float gD = exp(-2.20679 / (d + 3.91029) - 0.428934);
    float alpha = exp(3.62489 - 8.29288 / (d + 5.52825));
    float wD = exp(-0.599085 / (d - 0.641583) - 0.665888);
    return (1.0 - wD) * phase_h_g(c, gHG) + wD * phase_draine(c, alpha, gD);
  }

  static float3 sun_angles_to_direction(float3 deg) {
    // Convert degrees to radians
    float azimuth = radians(deg.x);
    float elevation = radians(deg.y);

    // Calculate direction components
    float x = sin(azimuth) * cos(elevation);
    float y = sin(elevation);
    float z = cos(azimuth) * cos(elevation);

    return float3(x, y, z);
  }

  static float2 ray_sphere_interesect(float3 r0, float3 rd, float3 s0,
                                      float sr) {
    // - r0: ray origin
    // - rd: normalized ray direction
    // - s0: sphere center
    // - sr: sphere radius
    // - Returns distance from r0 to first intersecion with sphere,
    //   or -1.0 if no intersection.
    float a = dot(rd, rd);
    float3 s0_r0 = r0 - s0;
    float b = 2.0 * dot(rd, s0_r0);
    float c = dot(s0_r0, s0_r0) - (sr * sr);
    if (b * b - 4.0 * a * c < 0.0) {
      return float2(-1.0, -1.0);
    }

    float t1 = (-b - sqrt((b * b) - 4.0 * a * c)) / (2.0 * a);
    float t2 = (-b + sqrt((b * b) - 4.0 * a * c)) / (2.0 * a);
    return float2(t1, t2);
  }
}

[shader("compute")]
[numthreads(8, 8, 1)]
void comp_main(uint2 px: SV_DispatchThreadID,
               uniform AtmosphereRendering atmosphere) {
  atmosphere.render(px);
}

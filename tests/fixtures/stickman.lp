{
  "format": "lp",
  "version": "0.1.0",
  "skeleton": {
    "bones": [
      { "name": "hip",   "parent": null, "length": 30.0,
        "setup": { "x": 128.0, "y": 80.0, "rotation": 1.5707963267948966, "scale_x": 1.0, "scale_y": 1.0, "shear_x": 0.0, "shear_y": 0.0 } },
      { "name": "spine", "parent": 0, "length": 40.0,
        "setup": { "x": 30.0, "y": 0.0, "rotation": 0.0, "scale_x": 1.0, "scale_y": 1.0, "shear_x": 0.0, "shear_y": 0.0 } },
      { "name": "head",  "parent": 1, "length": 20.0,
        "setup": { "x": 40.0, "y": 0.0, "rotation": 0.0, "scale_x": 1.0, "scale_y": 1.0, "shear_x": 0.0, "shear_y": 0.0 } },
      { "name": "arm_l", "parent": 1, "length": 25.0,
        "setup": { "x": 40.0, "y": 0.0, "rotation": 1.5707963267948966, "scale_x": 1.0, "scale_y": 1.0, "shear_x": 0.0, "shear_y": 0.0 } },
      { "name": "arm_r", "parent": 1, "length": 25.0,
        "setup": { "x": 40.0, "y": 0.0, "rotation": -1.5707963267948966, "scale_x": 1.0, "scale_y": 1.0, "shear_x": 0.0, "shear_y": 0.0 } },
      { "name": "leg_l", "parent": 0, "length": 35.0,
        "setup": { "x": 0.0, "y": -8.0, "rotation": 3.141592653589793, "scale_x": 1.0, "scale_y": 1.0, "shear_x": 0.0, "shear_y": 0.0 } },
      { "name": "leg_r", "parent": 0, "length": 35.0,
        "setup": { "x": 0.0, "y": 8.0, "rotation": 3.141592653589793, "scale_x": 1.0, "scale_y": 1.0, "shear_x": 0.0, "shear_y": 0.0 } },
      { "name": "cape_root", "parent": 1, "length": 18.0,
        "setup": { "x": 40.0, "y": 0.0, "rotation": 3.141592653589793, "scale_x": 1.0, "scale_y": 1.0, "shear_x": 0.0, "shear_y": 0.0 } },
      { "name": "cape_mid", "parent": 7, "length": 18.0,
        "setup": { "x": 18.0, "y": 0.0, "rotation": 0.0, "scale_x": 1.0, "scale_y": 1.0, "shear_x": 0.0, "shear_y": 0.0 } },
      { "name": "cape_tip", "parent": 8, "length": 18.0,
        "setup": { "x": 18.0, "y": 0.0, "rotation": 0.0, "scale_x": 1.0, "scale_y": 1.0, "shear_x": 0.0, "shear_y": 0.0 } }
    ]
  },
  "regions": [
    { "name": "head",  "bone": 2, "width": 20.0, "height": 20.0,
      "vertices": [
        { "local": [0.0,  -10.0], "weights": [[2, 1.0]] },
        { "local": [20.0, -10.0], "weights": [[2, 1.0]] },
        { "local": [20.0,  10.0], "weights": [[2, 1.0]] },
        { "local": [0.0,   10.0], "weights": [[2, 1.0]] }
      ]
    },
    { "name": "torso", "bone": 1, "width": 40.0, "height": 24.0,
      "vertices": [
        { "local": [0.0,  -12.0], "weights": [[1, 1.0]] },
        { "local": [40.0, -12.0], "weights": [[1, 1.0]] },
        { "local": [40.0,  12.0], "weights": [[1, 1.0]] },
        { "local": [0.0,   12.0], "weights": [[1, 1.0]] }
      ]
    },
    { "name": "arm_l", "bone": 3, "width": 25.0, "height": 6.0,
      "vertices": [
        { "local": [0.0,  -3.0], "weights": [[3, 1.0]] },
        { "local": [25.0, -3.0], "weights": [[3, 1.0]] },
        { "local": [25.0,  3.0], "weights": [[3, 1.0]] },
        { "local": [0.0,   3.0], "weights": [[3, 1.0]] }
      ]
    },
    { "name": "arm_r", "bone": 4, "width": 25.0, "height": 6.0,
      "vertices": [
        { "local": [0.0,  -3.0], "weights": [[4, 1.0]] },
        { "local": [25.0, -3.0], "weights": [[4, 1.0]] },
        { "local": [25.0,  3.0], "weights": [[4, 1.0]] },
        { "local": [0.0,   3.0], "weights": [[4, 1.0]] }
      ]
    },
    { "name": "leg_l", "bone": 5, "width": 35.0, "height": 6.0,
      "vertices": [
        { "local": [0.0,  -3.0], "weights": [[5, 1.0]] },
        { "local": [35.0, -3.0], "weights": [[5, 1.0]] },
        { "local": [35.0,  3.0], "weights": [[5, 1.0]] },
        { "local": [0.0,   3.0], "weights": [[5, 1.0]] }
      ]
    },
    { "name": "leg_r", "bone": 6, "width": 35.0, "height": 6.0,
      "vertices": [
        { "local": [0.0,  -3.0], "weights": [[6, 1.0]] },
        { "local": [35.0, -3.0], "weights": [[6, 1.0]] },
        { "local": [35.0,  3.0], "weights": [[6, 1.0]] },
        { "local": [0.0,   3.0], "weights": [[6, 1.0]] }
      ]
    },
    { "name": "cape", "bone": 7, "width": 54.0, "height": 16.0, "use_skin": true,
      "vertices": [
        { "local": [120.0, 150.0], "weights": [[7, 1.0]] },
        { "local": [136.0, 150.0], "weights": [[7, 1.0]] },
        { "local": [136.0, 132.0], "weights": [[7, 1.0]] },
        { "local": [120.0, 132.0], "weights": [[7, 1.0]] },
        { "local": [120.0, 132.0], "weights": [[8, 1.0]] },
        { "local": [136.0, 132.0], "weights": [[8, 1.0]] },
        { "local": [136.0, 114.0], "weights": [[8, 1.0]] },
        { "local": [120.0, 114.0], "weights": [[8, 1.0]] },
        { "local": [120.0, 114.0], "weights": [[9, 1.0]] },
        { "local": [136.0, 114.0], "weights": [[9, 1.0]] },
        { "local": [136.0, 96.0],  "weights": [[9, 1.0]] },
        { "local": [120.0, 96.0],  "weights": [[9, 1.0]] }
      ]
    }
  ],
  "animations": [
    {
      "name": "wave",
      "duration": 2.0,
      "timelines": [
        {
          "bone": 3,
          "property": "Rotate",
          "keyframes": [
            { "time": 0.0, "value": -0.8, "curve": {"kind":"Linear"} },
            { "time": 1.0, "value": 0.8, "curve": {"kind":"Linear"} },
            { "time": 2.0, "value": -0.8, "curve": {"kind":"Linear"} }
          ]
        },
        {
          "bone": 1,
          "property": "Rotate",
          "keyframes": [
            { "time": 0.0, "value": -0.15, "curve": {"kind":"Linear"} },
            { "time": 1.0, "value": 0.15, "curve": {"kind":"Linear"} },
            { "time": 2.0, "value": -0.15, "curve": {"kind":"Linear"} }
          ]
        }
      ]
    }
  ],
  "constraints": [
    { "type": "physics", "bone": 7, "bone_inertia": 0.6, "strength": 0.2,
      "damping": 0.88, "gravity": [0.0, -60.0], "mass": 1.0,
      "angle_min": -1.2, "angle_max": 1.2, "rotate_mix": 1.0 },
    { "type": "physics", "bone": 8, "bone_inertia": 0.7, "strength": 0.15,
      "damping": 0.85, "gravity": [0.0, -60.0], "mass": 1.0,
      "angle_min": -1.2, "angle_max": 1.2, "rotate_mix": 1.0 },
    { "type": "physics", "bone": 9, "bone_inertia": 0.8, "strength": 0.1,
      "damping": 0.82, "gravity": [0.0, -60.0], "mass": 1.0,
      "angle_min": -1.2, "angle_max": 1.2, "rotate_mix": 1.0 }
  ]
}

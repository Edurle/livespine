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
        "setup": { "x": 0.0, "y": 8.0, "rotation": 3.141592653589793, "scale_x": 1.0, "scale_y": 1.0, "shear_x": 0.0, "shear_y": 0.0 } }
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
  ]
}

{
  "format": "lp",
  "version": "0.1.0",
  "skeleton": {
    "bones": [
      { "name": "thigh", "parent": null, "length": 60.0,
        "setup": { "x": 128.0, "y": 200.0, "rotation": -1.5707963267948966, "scale_x": 1.0, "scale_y": 1.0, "shear_x": 0.0, "shear_y": 0.0 } },
      { "name": "shin",  "parent": 0, "length": 50.0,
        "setup": { "x": 60.0, "y": 0.0, "rotation": 0.0, "scale_x": 1.0, "scale_y": 1.0, "shear_x": 0.0, "shear_y": 0.0 } }
    ]
  },
  "regions": [
    { "name": "lower_leg", "bone": 1, "width": 20.0, "height": 50.0,
      "vertices": [
        { "local": [-10.0, 0.0],   "weights": [[1, 1.0]] },
        { "local": [10.0,  0.0],   "weights": [[1, 1.0]] },
        { "local": [10.0,  50.0],  "weights": [[1, 1.0]] },
        { "local": [-10.0, 50.0],  "weights": [[1, 1.0]] }
      ]
    }
  ],
  "constraints": [
    { "type": "ik",
      "bones": [0, 1],
      "target": [128.0, 90.0],
      "mix": 1.0,
      "bendDirection": 1,
      "softness": 0.0 }
  ]
}

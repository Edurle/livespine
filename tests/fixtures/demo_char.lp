{
  "format": "lp",
  "version": "0.1.0",
  "skeleton": {
    "bones": [
      { "name": "hip",   "parent": null, "length": 20.0,
        "setup": { "x": 128.0, "y": 128.0, "rotation": 0.0, "scale_x": 1.0, "scale_y": 1.0, "shear_x": 0.0, "shear_y": 0.0 } },
      { "name": "spine", "parent": 0,    "length": 30.0,
        "setup": { "x": 0.0,   "y": 0.0,   "rotation": 0.0, "scale_x": 1.0, "scale_y": 1.0, "shear_x": 0.0, "shear_y": 0.0 } },
      { "name": "head",  "parent": 1,    "length": 15.0,
        "setup": { "x": 0.0,   "y": 30.0,  "rotation": 0.0, "scale_x": 1.0, "scale_y": 1.0, "shear_x": 0.0, "shear_y": 0.0 } }
    ]
  },
  "regions": [
    { "name": "body", "bone": 1, "width": 60.0, "height": 80.0,
      "vertices": [
        { "local": [-30.0, -10.0], "weights": [[1, 1.0]] },
        { "local": [30.0,  -10.0], "weights": [[1, 1.0]] },
        { "local": [30.0,   70.0], "weights": [[1, 1.0]] },
        { "local": [-30.0,  70.0], "weights": [[1, 1.0]] }
      ]
    }
  ]
}

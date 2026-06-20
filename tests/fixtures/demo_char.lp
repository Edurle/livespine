{
  "format": "lp",
  "version": "0.1.0",
  "skeleton": {
    "bones": [
      { "name": "hip",   "parent": null, "length": 20.0,
        "setup": { "x": 128.0, "y": 100.0, "rotation": 1.5707963267948966, "scale_x": 1.0, "scale_y": 1.0, "shear_x": 0.0, "shear_y": 0.0 } },
      { "name": "spine", "parent": 0,    "length": 30.0,
        "setup": { "x": 20.0,  "y": 0.0,   "rotation": 0.0,                "scale_x": 1.0, "scale_y": 1.0, "shear_x": 0.0, "shear_y": 0.0 } },
      { "name": "head",  "parent": 1,    "length": 15.0,
        "setup": { "x": 30.0,  "y": 0.0,   "rotation": 0.0,                "scale_x": 1.0, "scale_y": 1.0, "shear_x": 0.0, "shear_y": 0.0 } }
    ]
  },
  "regions": [
    { "name": "body", "bone": 1, "width": 30.0, "height": 30.0,
      "vertices": [
        { "local": [0.0,  -15.0], "weights": [[1, 1.0]] },
        { "local": [30.0, -15.0], "weights": [[1, 1.0]] },
        { "local": [30.0,  15.0], "weights": [[1, 1.0]] },
        { "local": [0.0,   15.0], "weights": [[1, 1.0]] }
      ]
    }
  ],
  "animations": [
    {
      "name": "swing",
      "duration": 2.0,
      "timelines": [
        {
          "bone": 1,
          "property": "Rotate",
          "keyframes": [
            { "time": 0.0, "value": -0.6, "curve": {"kind":"Linear"} },
            { "time": 1.0, "value":  0.6, "curve": {"kind":"Linear"} },
            { "time": 2.0, "value": -0.6, "curve": {"kind":"Linear"} }
          ]
        }
      ]
    }
  ]
}

module.exports = {
    // presets: [
    //     [
    //         "@vue/app",
    //         {
    //             polyfills: [
    //                 "es6.promise",
    //                 "es6.symbol",
    //                 "es6.array.iterator",
    //                 "es6.object.assign",
    //             ],
    //             useBuiltIns: "entry",
    //         },
    //     ],
    // ],

    presets: [
        [ "@babel/preset-env", {
            "targets": {
                "esmodules": true,
                "ie": "11"
            }
        }]
    ]
}

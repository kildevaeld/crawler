const Path = require('path');


module.exports = {
    entry: {
        cheerio: "./cheerio.js",
        html_entities: "./html-entities.js",
        striptags: "./striptags.js",
        relaxed_json: './relaxed-json.js'
    },
    mode: 'none',
    output: {
        path: Path.join(__dirname, 'dist'),
        filename: "[name].js",
        libraryTarget: 'commonjs'
    },
    module: {
        rules: [{
            test: /\.m?js$/,
            exclude: /(node_modules|bower_components)/,
            use: {
                loader: 'babel-loader',
                options: {
                    presets: ['@babel/preset-env']
                }
            }
        }]
    }

}
const colors = require('tailwindcss/colors')

module.exports = {
    content: [
        "index.html",
        "./src/**/*.rs"
    ],
    theme: {
        extend: {
            // Our application colour palette is defined here.
            colors: {
                'base': colors.gray,
                'primary': colors.blue,
                'secondary': colors.yellow,
                'error': colors.red
            }
        }
    },
    plugins: [],
}
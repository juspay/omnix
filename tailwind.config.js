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
                'primary': colors.pink,
                'secondary': colors.blue,
                'error': colors.red
            }
        }
    },
    plugins: [],
}
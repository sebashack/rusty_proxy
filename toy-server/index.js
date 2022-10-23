const express = require('express')
const app = express()
const bodyParser = require('body-parser');

app.use(bodyParser.json());
app.use(express.urlencoded({extended:false}));
app.use(express.static('assets'));

app.get('/query_params', function (req, res) {
  console.log(req.query)
  res.send('QUERY PARAMS OK')
})

app.put('/json_put', function (req, res) {
  console.log(JSON.stringify(req.headers))
  console.log('-----')
  console.log(JSON.stringify(req.body))
  res.send('JSON BODY RECEIVED OK')
})

app.post('/url_post', function (req, res) {
  console.log(JSON.stringify(req.headers))
  console.log('-----')
  console.log(req.body)
  res.send('URL ENCODED OK')
})

console.log("Running on port " + process.env.PORT)
app.listen(process.env.PORT)

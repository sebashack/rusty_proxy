const express = require('express')
const app = express()
const bodyParser = require('body-parser');

app.use(bodyParser.json());
app.use(express.urlencoded({extended:false}));

app.get('/', function (req, res) {
  console.log(JSON.stringify(req.headers))
  res.send('HOME OK')
})

app.get('/query_params', function (req, res) {
  console.log(req.query)
  res.send('QUERY OK')
})

app.put('/json_put', function (req, res) {
  console.log(JSON.stringify(req.headers))
  console.log('-----')
  console.log(JSON.stringify(req.body))
  res.send('JSON RECEIVED OK')
})

app.post('/url_post', function (req, res) {
  console.log(JSON.stringify(req.headers))
  console.log('-----')
  console.log(req.body)
  res.send('URL ENCODED OK')
})

app.get('/vanh_gogh.jpg', function (req, res) {
  console.log(JSON.stringify(req.headers))
  res.sendFile('/home/sebastian/university/networking/toy-server/images/vanh_gogh.jpg')
})

console.log("Running on port 3000")
app.listen(3000)

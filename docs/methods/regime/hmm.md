# Discrete HMM decoding

`DiscreteHmm::viterbi` validates normalized non-negative initial, transition, and emission distributions, then applies log-domain Viterbi decoding to discrete observation symbols. Impossible observations return an error.

Only decoding is implemented. There is no Baum–Welch training, continuous emission distribution, posterior smoothing, or model-order selection.

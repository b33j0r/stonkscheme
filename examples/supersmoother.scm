;; ─────────────────────────────────────────────────────────────
;; Supersmoother Filter in StonkScheme
;; ─────────────────────────────────────────────────────────────

(inputs
	(price (Array Price))
	(period Duration)
	)

(vars
	(output (Array Price) [])
	)

;; Compute smoothing coefficients from period
(define (compute-supersmoother-coefficients period)
	(let (
				 (k1 (/ (* -1.1414 3.14159) period))
				 (k2 (/ (* 1.414 180) period))
				 (a1 (expvalue k1))
				 (b1 (* 2 (cosine k2)))
				 (c2 b1)
				 (c3 (negate (* a1 a1)))
				 (c1 (- 1 c2 c3))
				 )
		(tuple c1 c2 c3)
		)
	)

;; Apply smoothing at index `n` using the coefficients
(define (supersmooth n c1 c2 c3)
	(if (< n 2)
		(begin
			(set-array output n (get-array price n))
			(get-array price n))
		(let (
					 (avg (/ (+ (get-array price n) (get-array price (- n 1))) 2))
					 (smoothed
						 (+ (* c1 avg)
							 (* c2 (get-array output (- n 1)))
							 (* c3 (get-array output (- n 2)))))
					 )
			(set-array output n smoothed)
			smoothed
			)
		)
	)

;; Run the full filter over the input array
(define (run-filter)
	(let ((c1 c2 c3) (compute-supersmoother-coefficients period))
		(loop ((i 0) (< i (length price)) (+ i 1))
			(supersmooth i c1 c2 c3)
			)
		)
	)

;; Entry point for the program
(define (main)
	;; load price data from CSV file
	(set! price (load-csv "prices.csv"))       ; CSV should be just a column of floats
	(set! period 10)                            ; set default smoothing period

	(run-filter)

	(plot output
		:title "Supersmoother Output"
		:x-label "Time"
		:y-label "Price"
		:color "steelblue")
	)

(main)

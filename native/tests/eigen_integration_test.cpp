#include <catch2/catch_test_macros.hpp>
#include <Eigen/Dense>
#include <cmath>

TEST_CASE("Eigen Integration - Basic Matrix Operations", "[eigen]")
{
  // Test basic matrix creation and operations
  Eigen::MatrixXd m(2, 2);
  m(0, 0) = 3.0;
  m(1, 0) = 2.5;
  m(0, 1) = -1.0;
  m(1, 1) = m(1, 0) + m(0, 1);

  REQUIRE(m(0, 0) == 3.0);
  REQUIRE(m(1, 0) == 2.5);
  REQUIRE(m(0, 1) == -1.0);
  REQUIRE(m(1, 1) == 1.5);
}

TEST_CASE("Eigen Integration - Matrix Multiplication", "[eigen]")
{
  // Test matrix multiplication (useful for portfolio optimization)
  Eigen::MatrixXd A(2, 2);
  A << 1, 2, 3, 4;

  Eigen::MatrixXd B(2, 2);
  B << 5, 6, 7, 8;

  Eigen::MatrixXd C = A * B;

  REQUIRE(C(0, 0) == 19.0);  // 1*5 + 2*7
  REQUIRE(C(0, 1) == 22.0);  // 1*6 + 2*8
  REQUIRE(C(1, 0) == 43.0);  // 3*5 + 4*7
  REQUIRE(C(1, 1) == 50.0);  // 3*6 + 4*8
}

TEST_CASE("Eigen Integration - Vector Operations", "[eigen]")
{
  // Test vector operations (useful for portfolio weights)
  Eigen::VectorXd v(3);
  v << 0.4, 0.3, 0.3;

  // Test vector sum (should equal 1.0 for portfolio weights)
  double sum = v.sum();
  REQUIRE(std::abs(sum - 1.0) < 1e-10);

  // Test vector norm
  double norm = v.norm();
  REQUIRE(norm > 0.0);
}

TEST_CASE("Eigen Integration - Linear System Solver", "[eigen]")
{
  // Test solving linear systems (useful for portfolio optimization)
  Eigen::MatrixXd A(2, 2);
  A << 2, 1, 1, 2;

  Eigen::VectorXd b(2);
  b << 3, 3;

  Eigen::VectorXd x = A.colPivHouseholderQr().solve(b);

  REQUIRE(std::abs(x(0) - 1.0) < 1e-10);
  REQUIRE(std::abs(x(1) - 1.0) < 1e-10);
}

TEST_CASE("Eigen Integration - C++20 Compatibility", "[eigen]")
{
  // Verify Eigen works with C++20 features
  auto create_matrix = []() -> Eigen::MatrixXd {
    Eigen::MatrixXd m(2, 2);
    m << 1, 2, 3, 4;
    return m;
  };

  auto m = create_matrix();
  REQUIRE(m.rows() == 2);
  REQUIRE(m.cols() == 2);
  REQUIRE(m(0, 0) == 1.0);
}

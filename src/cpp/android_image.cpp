#if defined(__ANDROID__)
#include <mln/util/image.hpp>
#include <mln/util/premultiply.hpp>
#include <stdexcept>
extern "C" bool mln_rust_decode_image(const uint8_t*, size_t, void*, void (*)(void*, uint32_t, uint32_t, const uint8_t*, size_t));
namespace {
void receiveImage(void* context, uint32_t width, uint32_t height, const uint8_t* bytes, size_t size) {
    auto& image = *static_cast<mln::UnassociatedImage*>(context);
    image = mln::UnassociatedImage({width, height}, bytes, size);
}
}
namespace mln {
PremultipliedImage decodeImage(const std::string& data) {
    UnassociatedImage image;
    if (!mln_rust_decode_image(reinterpret_cast<const uint8_t*>(data.data()), data.size(), &image, receiveImage)) {
        throw std::runtime_error("Could not decode map image");
    }
    return util::premultiply(std::move(image));
}
}
#endif
